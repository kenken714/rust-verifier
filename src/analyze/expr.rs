use rustc_ast::ast::LitKind;
use rustc_hir::Lit;
use rustc_middle::mir::BinOp;

use crate::analyze::core::AnalysisType;
use crate::analyze::*;
use crate::thir::rthir::{RExprKind::*, RPatKind::*, RThir};

impl<'tcx> Analyzer<'tcx> {
    pub fn analyze_literal(
        &self,
        expr: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<String, AnalysisError> {
        // とりあえずInt literalのみ
        if let Literal { lit, neg } = &expr.kind {
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
        Ok(()) //not implemented
    }
    pub fn op_to_const(&self, op: BinOp, lhs: &str, rhs: &str) -> Result<String, AnalysisError> {
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
                Ok(self.op_to_const(*op, &lhs, &rhs)?)
            }
            _ => Err(AnalysisError::Unsupported(
                "Unsupported expression".to_string(),
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
                "Only block expressions are supported".to_string(),
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
                _ => unreachable!(),
            }
        } else {
            Err(AnalysisError::Unsupported("Unknown extern function".into()))
        }
    }
}
