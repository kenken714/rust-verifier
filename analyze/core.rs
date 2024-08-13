use crate::analyze::*;

impl<'tcx> Analyzer<'tcx> {
    pub fn analyze(&self, rthir: Rc<Rthir<'tcx>>) -> Result<(), AnalysisError> {
        if let Some(body) = &rthir.body {
            let main_env = Env::new();
            self.analyze_body((*body).clone(), main_env)?
        }
        Ok(())
    }

    fn analyze_body(&self, body: Rc<RExpr<'tcx>>, env: Env<'tcx>) -> Result<(), AnalysisError> {
        if let RExprKind::Block { stmts, expr } = &body.kind {
            let mut stmts = stmts.clone().into_iter().peekable();
            while let Some(stmt) = stmts.next() {
                let res = self.analyze_expr(stmt.clone(), env)?;
                match res {
                    AnalysisType::Invariant(expr) => {
                        self.analyze_expr(expr.clone(), env)?; //loop is currently not supported
                    }
                    AnalysisType::Break => Break,
                    AnalysisType::Other => (),
                }
            }
            if let Some(expr) = expr {
                self.analyze_expr(expr.clone(), env)?;
            }
            Ok(())
        }
        Unsupported("Only block expressions are supported".to_string())
    }

    fn analyze_expr(
        &self,
        expr: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        use RExprKind::*;
        let mut res = AnalysisType::Other;
        match expr.kind.clone {
            Literal { expr } => {
                self.analyze_literal(expr, env);
            }
            Binary { op, lhs, rhs } => {
                let lhs = self.analyze_expr(lhs.clone(), env);
                let rhs = self.analyze_expr(rhs.clone(), env);
                self.op_to_const(op, lhs, rhs);
            }
            _ => {
                return Err(AnalysisError::Unsupported(
                    "Unsupported expression".to_string(),
                ))
            }
        }
        Ok(res)
    }
}

pub enum AnalysisType {
    Invariant(Rc<RExpr<'tcx>>),
    Break,
    Other,
}
pub enum AnalysisError {
    Unsupported(String),
    Unimplemented(String),
}
