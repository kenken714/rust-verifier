use rustc_middle::thir::LocalVarId;
use rustc_middle::ty::{self, Ty};
use rustc_span::Span;
use std::rc::Rc;

use std::collections::{HashMap, VecDeque};

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

    pub fn len(&self) -> usize {
        self.path.len()
    }

    pub fn add_smt_command(&mut self, constraint: String, expr: Rc<RExpr<'tcx>>) {
        self.path.push(Lir::new_assert(constraint, expr, None));
    }

    pub fn get_smt_commands(&self) -> Result<String, AnalysisError> {
        let smt_str = self
            .path
            .iter()
            .map(|smt_command| smt_command.to_smt().unwrap())
            .collect::<Vec<String>>();
        Ok(smt_str.join("\n"))
    }

    pub fn get_smt_command(&self, idx: usize) -> Result<String, AnalysisError> {
        self.path
            .get(idx)
            .ok_or(AnalysisError::OutOfBounds(idx))
            .and_then(|smt_command| Ok(smt_command.to_smt()?))
    }

    //TODO: fix 仮で書いている
    pub fn get_smt_command_for_assume(&self) -> Result<String, AnalysisError> {
        let len = self.path.len();
        let mut command = String::new();
        for i in 0..(len - 1) {
            if let LirKind::Assume(_) = self.path[i].kind {
                command.push_str(&self.path[i].to_smt()?);
            }
        }
        command.push_str(&self.path[len - 1].to_assert()?);
        Ok(command)
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
        //TODO: fix
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
}
