use rustc_middle::thir::LocalVarId;
use rustc_middle::ty::{Ty, TyKind};
use rustc_span::Span;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;

use std::collections::HashMap;

use crate::analyze::core::AnalysisError;
use crate::analyze::lir::Lir;
use crate::analyze::Analyzer;
use crate::analyze::LirKind;
use crate::analyze::RExpr;

pub struct Env<'tcx> {
    pub name: String,
    pub path: Vec<Lir<'tcx>>,
    pub vars: Vec<(Ty<'tcx>, String)>,
    pub env_map: HashMap<LocalVarId, Lir<'tcx>>,
}

impl<'tcx> Env<'tcx> {
    pub fn new() -> Self {
        Self {
            name: String::from("main"),
            path: Vec::new(),
            env_map: HashMap::new(),
            vars: Vec::new(),
        }
    }

    pub fn from(
        name: String,
        path: Vec<Lir<'tcx>>,
        env_map: HashMap<LocalVarId, Lir<'tcx>>,
        vars: Vec<(Ty<'tcx>, String)>,
    ) -> Self {
        Self {
            name,
            path,
            env_map,
            vars,
        }
    }

    pub fn verify_z3(&self, assert: String, span: Span) -> Result<(), AnalysisError> {
        let mut command = Command::new("z3")
            .arg("-in")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn z3 process");
        let mut smt_str = String::new();

        smt_str.push_str(&self.get_smt_commands()?);
        smt_str.push_str(format!("\n(assert (not {}))\n", assert).as_str());
        smt_str.push_str("\n(check-sat)\n");

        let stdin = command.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(smt_str.as_bytes())
            .expect("Failed to write to stdin");
        drop(stdin);

        let output = command.wait_with_output().expect("Failed to read stdout");
        let output_str = String::from_utf8(output.stdout).expect("Failed to convert to string");

        println!("SMT: \n {}", smt_str);
        println!("Output: \n {}", output_str);
        if output_str.contains("unsat") {
            println!("Verification succeeded :)");
            Ok(())
        } else {
            Err(AnalysisError::VerificationFailed)
        }
    }

    pub fn len(&self) -> usize {
        self.path.len()
    }

    pub fn add_smt_command(&mut self, constraint: String, expr: Rc<RExpr<'tcx>>) {
        self.path.push(Lir::new_assume(constraint, expr, None));
    }

    pub fn get_smt_commands(&self) -> Result<String, AnalysisError> {
        let smt_var_str = self
            .vars
            .iter()
            .map(|smt_var| self.var_to_smt(smt_var).unwrap())
            .collect::<Vec<String>>();
        let smt_str = self
            .path
            .iter()
            .map(|smt_command| self.path_to_smt(smt_command).unwrap())
            .collect::<Vec<String>>();
        Ok(format!(
            "{}\n{}",
            smt_var_str.join("\n"),
            smt_str.join("\n")
        ))
    }

    pub fn var_to_smt(&self, var: &(Ty<'tcx>, String)) -> Result<String, AnalysisError> {
        let (ty, name) = var;
        match ty.kind() {
            TyKind::Bool => Ok(format!("(declare-const {} Bool)", name)),
            TyKind::Int(_) => Ok(format!("(declare-const {} Int)", name)),
            TyKind::Float(_) => Ok(format!("(declare-const {} Real)", name)),
            _ => Err(AnalysisError::Unsupported(
                "Unsupported variable type".to_string(),
            )),
        }
    }
    pub fn path_to_smt(&self, path: &Lir<'tcx>) -> Result<String, AnalysisError> {
        use LirKind::*;

        match &path.kind {
            //Assert(constraint) => Ok(format!("(assert (not {}))", constraint)),
            Assume(constraint) => Ok(format!("(assert {})", constraint)),
            _ => Err(AnalysisError::Unsupported(
                "Unsupported annotation kind".to_string(),
            )),
        }
    }

    pub fn add_random_var(&mut self, ty: Ty<'tcx>, name: String) {
        self.vars.push((ty, name));
    }

    pub fn add_param(
        &mut self,
        name: String,
        ty: Ty<'tcx>,
        var_id: LocalVarId,
        pat: Rc<RExpr<'tcx>>,
    ) {
        self.env_map.insert(
            var_id,
            Lir::new_param(name.clone(), ty.clone(), pat, Some(String::new())),
        );
    }

    pub fn assign_value(&mut self, var_id: LocalVarId, constraint: String, expr: Rc<RExpr<'tcx>>) {
        let var = self
            .env_map
            .get_mut(&var_id)
            .expect("assign failed; target variable not found");
        var.assume = Some(constraint);
    }

    pub fn new_env_from_str(&self, name: String, span: Span) -> Result<Env<'tcx>, AnalysisError> {
        let name = self.get_unique_name(name, span);
        Ok(Env::from(
            name,
            self.path.clone(),
            self.env_map.clone(),
            self.vars.clone(),
        ))
    }

    pub fn get_unique_name(&self, name: String, span: Span) -> String {
        let span_str = Analyzer::get_name_from_span(span);
        format!("{}_{}", name, span_str)
    }

    pub fn merge_env(&mut self, cond: &String, then_env: Env<'tcx>, else_env: Option<Env<'tcx>>) {
        let mut new_env_map = HashMap::new();
        let mut current_env_map = self.env_map.clone();
        match else_env {
            Some(env) => {
                for (var_id, lir) in current_env_map.iter_mut() {
                    let then_lir = then_env.env_map.get(var_id);
                    let else_lir = env.env_map.get(var_id);
                    if let (Some(then_lir), Some(else_lir)) = (then_lir, else_lir) {
                        let then_constraint = then_lir.assume.clone().unwrap();
                        let else_constraint = else_lir.assume.clone().unwrap();
                        let constraint =
                            format!("(ite {} {} {})", cond, then_constraint, else_constraint);
                        new_env_map.insert(
                            *var_id,
                            Lir::new(lir.kind.clone(), lir.expr.clone(), Some(constraint)),
                        );
                    }
                }
            }
            None => {
                for (var_id, lir) in current_env_map.iter_mut() {
                    let then_lir = then_env.env_map.get(var_id);
                    if let Some(then_lir) = then_lir {
                        let then_constraint = then_lir.assume.clone().unwrap();
                        let constraint = format!(
                            "(ite {} {} {})",
                            cond,
                            then_constraint,
                            lir.assume.clone().unwrap()
                        );
                        new_env_map.insert(
                            *var_id,
                            Lir::new(lir.kind.clone(), lir.expr.clone(), Some(constraint)),
                        );
                    }
                }
            }
        }
        self.env_map = new_env_map;
    }

    pub fn adapt_cond(&mut self, cond: &String, path: &Vec<Lir<'tcx>>) {
        /*
        println!("adapt_cond: {}", cond);
        for lir in path.iter() {
            println!("lir: {:?}", lir.assume.as_ref().expect("assume not exist"));
        }
        for lir in self.path.iter() {
            println!("lir: {:?}", lir.assume.as_ref().expect("assume not exist"));
        }*/
        for i in path.len()..self.len() {
            self.path[i] = Lir::new_assume(
                cond.clone(),
                self.path[i].expr.clone(),
                self.path[i].assume.clone(),
            );
        }
    }
    pub fn merge_ite_env(
        &mut self,
        cond: &String,
        mut then_env: Env<'tcx>,
        mut else_env: Option<Env<'tcx>>,
    ) -> Result<(), AnalysisError> {
        then_env.adapt_cond(&cond, &self.path);
        if let Some(env) = else_env.as_mut() {
            env.adapt_cond(&format!("(not {})", cond), &self.path);
        }
        self.merge_env(&cond, then_env, else_env);
        Ok(())
    }
}
