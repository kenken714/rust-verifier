use std::rc::Rc;

use rustc_ast::ast::LitKind;
use rustc_hir::Lit;
use rustc_middle::mir::{BinOp, UnOp};
use rustc_middle::thir::*;
use rustc_middle::ty::{Ty, TyKind};

use crate::analyze::core::{AnalysisError, AnalysisType};
use crate::analyze::Analyzer;
use crate::analyze::Env;
use crate::thir::rthir::*;

impl<'tcx> Analyzer<'tcx> {
    pub fn analyze_literal(
        &self,
        expr: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        // とりあえずInt literalのみ
        if let RExprKind::Literal { lit, neg } = &expr.kind {
            if let LitKind::Int(i, _) = lit.node {
                Ok(if *neg {
                    format!("-{}", i)
                } else {
                    format!("{}", i)
                })
            } else {
                Err(AnalysisError::Unsupported(
                    "Only Int literals are supported".to_string(),
                ))
            }
        } else {
            Err(AnalysisError::Unsupported(
                "Only literals are supported".to_string(),
            ))
        }
    }

    pub fn analyze_binary(
        &self,
        expr: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<(), AnalysisError> {
        let const_expr = self.expr_to_const(expr.clone(), env)?;
        env.add_smt_command(const_expr, expr.clone());
        Ok(())
    }

    pub fn analyze_params(
        &self,
        params: &Vec<RParam<'tcx>>,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<(), AnalysisError> {
        use RExprKind::*;
        use RPatKind::*;
        for (param, arg) in params.iter().zip(args.iter()) {
            if let Some(pat) = &param.pat {
                if let RExpr {
                    kind: Pat { kind }, ..
                } = pat.as_ref()
                {
                    match kind {
                        Binding { ty, var, .. } => {
                            let name = Analyzer::get_name_from_span(pat.span);
                            env.add_param(name.clone(), ty.clone(), *var, pat.clone());
                            let arg_str = self.expr_to_const(arg.clone(), env)?;
                            env.assign_value(*var, arg_str, arg.clone());
                        }
                        _ => {
                            return Err(AnalysisError::Unsupported(
                                "Unsupported pattern in parameter".to_string(),
                            ))
                        }
                    }
                } else {
                    return Err(AnalysisError::Unsupported(
                        "Unsupported expression in parameter".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn binop_to_const(&self, op: BinOp, lhs: &str, rhs: &str) -> Result<String, AnalysisError> {
        use BinOp::*;
        let bin_op = match op {
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            Rem => "%",
            Eq => "=",
            Ne => "!=",
            Lt => "<",
            Le => "<=",
            Gt => ">",
            Ge => ">=",
            _ => {
                return Err(AnalysisError::Unsupported(
                    "Unsupported operator".to_string(),
                ))
            }
        };
        Ok(format!("({} {} {})", bin_op, lhs, rhs))
    }

    pub fn analyze_let_stmt(
        &self,
        pattern: Rc<RExpr<'tcx>>,
        init: Option<Rc<RExpr<'tcx>>>,
        else_block: Option<Rc<RExpr<'tcx>>>,
        env: &mut Env<'tcx>,
    ) -> Result<(), AnalysisError> {
        if let RExprKind::Pat { kind, .. } = &pattern.kind {
            match kind {
                RPatKind::Binding { ty, var, .. } => {
                    let name = Analyzer::get_name_from_span(pattern.span);
                    env.add_param(name.clone(), ty.clone(), *var, pattern.clone());

                    if let Some(init) = init {
                        match self.expr_to_const(init.clone(), env) {
                            Ok(str) => {
                                env.assign_value(*var, str, init.clone());
                            }
                            Err(e) => match e {
                                AnalysisError::RandFunctions => {
                                    return Err(AnalysisError::Unimplemented(
                                        "Random functions are currently not supported".to_string(),
                                    ));
                                }
                                _ => return Err(e),
                            },
                        }
                    }
                }
                _ => {
                    return Err(AnalysisError::Unsupported(
                        format!("Unsupported pattern in let statement {:?}", kind).to_string(),
                    ))
                }
            }
        } else {
            return Err(AnalysisError::Unsupported(
                "Unsupported expression in let statement".to_string(),
            ));
        }
        if let Some(else_block) = else_block {
            self.analyze_body(else_block.clone(), env)?;
        }
        Ok(())
    }

    pub fn analyze_assign(
        &self,
        lhs: Rc<RExpr<'tcx>>,
        rhs: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<(), AnalysisError> {
        let rhs_str = self.expr_to_const(rhs.clone(), env)?;
        let var = env
            .env_map
            .get_mut(&Analyzer::expr_to_var_id(lhs))
            .expect("assign target variant not found");
        var.assume = Some(rhs_str);
        Ok(())
    }

    pub fn logical_op_to_const(
        &self,
        op: LogicalOp,
        lhs: &str,
        rhs: &str,
    ) -> Result<String, AnalysisError> {
        use LogicalOp::*;
        let logical_op = match op {
            And => "and",
            Or => "or",
        };
        Ok(format!("({} {} {})", logical_op, lhs, rhs))
    }

    pub fn unop_to_const(&self, op: UnOp, arg: &str) -> Result<String, AnalysisError> {
        use rustc_middle::mir::UnOp::*;
        let un_op = match op {
            Not => "not",
            Neg => "-",
            _ => {
                return Err(AnalysisError::Unsupported(
                    "Unsupported operator in unary expression".to_string(),
                ))
            }
        };
        Ok(format!("({} {})", un_op, arg))
    }

    pub fn var_ref_to_const(
        &self,
        id: LocalVarId,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        Ok(env
            .env_map
            .get(&id)
            .expect(format!("Variable not found: {:?}", id).as_str())
            .assume
            .clone()
            .unwrap_or_default())
    }
    pub fn if_to_const(
        &self,
        cond: Rc<RExpr<'tcx>>,
        then: Rc<RExpr<'tcx>>,
        else_opt: Option<Rc<RExpr<'tcx>>>,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        let mut cond_env = env.new_env_from_str("cond".to_string(), cond.span)?;
        let cond_str = self.expr_to_const(cond.clone(), env)?;
        cond_env.add_smt_command(cond_str.clone(), cond.clone());

        let mut then_env = env.new_env_from_str("then".to_string(), then.span)?;
        let then_str = self.expr_to_const(then.clone(), env)?;
        then_env.add_smt_command(then_str.clone(), then.clone());

        let else_expr = else_opt.expect("No else expression in if statement");
        let mut else_env = env.new_env_from_str("else".to_string(), else_expr.span)?;
        let else_str = self.expr_to_const(else_expr.clone(), env)?;
        else_env.add_smt_command(else_str.clone(), else_expr.clone());

        Ok(format!("(ite {} {} {})", cond_str, then_str, else_str))
    }

    pub fn expr_to_const(
        &self,
        expr: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        use RExprKind::*;
        match &expr.kind {
            Literal { lit, neg } => Ok(self.literal_to_const(lit, *neg)?),
            Binary { op, lhs, rhs } => {
                let lhs = self.expr_to_const(lhs.clone(), env)?;
                let rhs = self.expr_to_const(rhs.clone(), env)?;
                Ok(self.binop_to_const(*op, &lhs, &rhs)?)
            }
            LogicalOp { op, lhs, rhs } => {
                let lhs = self.expr_to_const(lhs.clone(), env)?;
                let rhs = self.expr_to_const(rhs.clone(), env)?;
                Ok(self.logical_op_to_const(*op, &lhs, &rhs)?)
            }
            Unary { op, arg } => {
                let arg = self.expr_to_const(arg.clone(), env)?;
                Ok(self.unop_to_const(*op, &arg)?)
            }
            //Call { ty, args, .. } => Ok(self.fn_to_expr(*ty, args.clone(), expr.clone(), env)?),
            If {
                cond,
                then,
                else_opt,
            } => Ok(self.if_to_const(cond.clone(), then.clone(), else_opt.clone(), env)?),
            Block { .. } => self.block_to_const(expr.clone(), env),
            VarRef { id } => self.var_ref_to_const(*id, env),
            Call { ty, args, .. } => Ok(self
                .fn_to_const(*ty, args.clone(), expr.clone(), env)?
                .to_string()),
            _ => Err(AnalysisError::Unsupported(
                format!("Unsupported expression {:?}", expr.kind).to_string(),
            )),
        }
    }

    pub fn block_to_const(
        &self,
        block: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        let mut res = String::new();
        if let RExprKind::Block { stmts, expr } = &block.kind {
            if let Some(expr) = expr {
                res = self.expr_to_const(expr.clone(), env)?
            }
        } else {
            return Err(AnalysisError::Unsupported(
                "Unknown block expression".to_string(),
            ));
        }
        Ok(res)
    }

    pub fn literal_to_const(&self, lit: &Lit, neg: bool) -> Result<String, AnalysisError> {
        if let LitKind::Int(i, _) = lit.node {
            Ok(if neg {
                format!("-{}", i)
            } else {
                format!("{}", i)
            })
        } else {
            Err(AnalysisError::Unsupported(
                "Only Int literals are supported".to_string(),
            ))
        }
    }

    pub fn analyze_if(
        &self,
        cond: Rc<RExpr<'tcx>>,
        then: Rc<RExpr<'tcx>>,
        else_opt: Option<Rc<RExpr<'tcx>>>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        let cond_env = env.new_env_from_str("cond".to_string(), cond.span);
        let cond_str = self.expr_to_const(cond.clone(), env)?;
        env.add_smt_command(cond_str.clone(), cond.clone());

        let then_env = env.new_env_from_str("then".to_string(), then.span);
        let then_str = self.expr_to_const(then.clone(), env)?;
        env.add_smt_command(then_str.clone(), then.clone());

        let else_opt = if let Some(else_expr) = else_opt {
            self.expr_to_const(else_expr.clone(), env)?
        } else {
            String::new()
        };
        Ok(AnalysisType::Other)
    }

    pub fn analyze_fn(
        &self,
        ty: Ty<'tcx>,
        args: Box<[Rc<RExpr<'tcx>>]>,
        body: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        match ty.kind() {
            TyKind::FnDef(def_id, ..) => {
                let mut fn_info = self.get_fn_info(def_id);
                if let Some(fn_thir) = self.get_local_fn(def_id) {
                    self.analyze_local_fn(fn_thir, args, env)
                } else {
                    self.analyze_annotate_fn(fn_info, args, env)
                }
            }
            _ => return Err(AnalysisError::Unsupported("FnDef is not found".to_string())),
        }
    }

    pub fn analyze_local_fn(
        &self,
        rthir: Rc<RThir<'tcx>>,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        self.analyze_params(&rthir.params, args, env)?;
        if let Some(body) = &rthir.body {
            self.analyze_body((*body).clone(), env)?;
        } else {
            return Err(AnalysisError::Unsupported(
                "No RThir body Found".to_string(),
            ));
        }
        Ok(AnalysisType::Other)
    }

    pub fn analyze_annotate_fn(
        &self,
        fn_info: Vec<String>,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        if fn_info[0] == "verify_modules" {
            match fn_info[1].as_str() {
                "Vassert" => self.analyze_assert(args, env),
                "Vassume" => self.analyze_assume(args, env),
                "Vinvariant" => self.analyze_invariant(args, env),
                _ => unreachable!(),
            }
        } else {
            Err(AnalysisError::Unsupported("Unknown extern function".into()))
        }
    }

    pub fn fn_to_const(
        &self,
        ty: Ty<'tcx>,
        args: Box<[Rc<RExpr<'tcx>>]>,
        body: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        match ty.kind() {
            TyKind::FnDef(def_id, ..) => {
                let mut fn_info = self.get_fn_info(def_id);
                if let Some(fn_thir) = self.get_local_fn(def_id) {
                    self.local_fn_to_const(fn_thir, args, env)
                } else {
                    Err(AnalysisError::Unimplemented(
                        ("Extern functions to const are not supported".to_string()),
                    ))
                    //self.annotate_fn_to_const(fn_info, args, env)
                }
            }
            _ => return Err(AnalysisError::Unsupported("FnDef is not found".to_string())),
        }
    }

    pub fn local_fn_to_const(
        &self,
        rthir: Rc<RThir<'tcx>>,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        self.analyze_params(&rthir.params, args, env)?;
        if let Some(body) = &rthir.body {
            self.analyze_body((*body).clone(), env)?;
        } else {
            return Err(AnalysisError::Unsupported(
                "No RThir body Found".to_string(),
            ));
        }
        Ok(String::new())
    }

    /*WIP

    pub fn annotate_fn_to_const(
        &self,
        fn_info: Vec<String>,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        if fn_info[0] == "verify_modules" {
            match fn_info[1].as_str() {
                "Vassert" => self.analyze_assert(args, env),
                "Vassume" => self.analyze_assume(args, env),
                "Vinvariant" => self.analyze_invariant(args, env),
                _ => unreachable!(),
            }
        } else {
            Err(AnalysisError::Unsupported("Unknown extern function".into()))
        }
    }
    */
}
