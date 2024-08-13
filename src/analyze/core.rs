use crate::analyze::*;
use crate::thir::rthir::RThir;

impl<'tcx> Analyzer<'tcx> {
    pub fn analyze_enter(&self, rthir: Rc<RThir<'tcx>>) -> Result<(), AnalysisError> {
        if let Some(body) = &rthir.body {
            let mut main_env = Env::new();
            self.analyze_body((*body).clone(), &mut main_env)?
        }
        Ok(())
    }

    fn analyze_body(
        &self,
        body: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<(), AnalysisError> {
        if let RExprKind::Block { stmts, expr } = &body.kind {
            let mut stmts = stmts.clone().into_iter().peekable();
            while let Some(stmt) = stmts.next() {
                let res = self.analyze_expr(stmt.clone(), env)?;
                match res {
                    AnalysisType::Invariant(expr) => {
                        self.analyze_expr(expr.clone(), env)?; //loop is currently not supported
                    }
                    AnalysisType::Break => break,
                    AnalysisType::Other => (),
                }
            }
            if let Some(expr) = expr {
                self.analyze_expr(expr.clone(), env)?;
            }
            return Ok(());
        }
        Err(AnalysisError::Unsupported(
            "Only block expressions are supported".to_string(),
        ))
    }

    fn analyze_expr(
        &self,
        expr: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        use RExprKind::*;
        let res = AnalysisType::Other;
        match expr.kind.clone() {
            Literal { .. } => {
                self.analyze_literal(expr, env);
            }
            Binary { .. } => {
                self.analyze_binary(expr, env);
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

#[derive(Debug)]
pub enum AnalysisType<'tcx> {
    Invariant(Rc<RExpr<'tcx>>),
    Break,
    Other,
}

#[derive(Debug)]
pub enum AnalysisError {
    Unsupported(String),
    Unimplemented(String),
}
