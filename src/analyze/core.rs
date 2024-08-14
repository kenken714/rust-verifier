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

    pub fn analyze_body(
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
                    AnalysisType::Return(_) => break,
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

    pub fn analyze_expr(
        &self,
        expr: Rc<RExpr<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        use RExprKind::*;
        let mut res = AnalysisType::Other;
        match expr.kind.clone() {
            Literal { .. } => {
                self.analyze_literal(expr, env)?;
            }
            Binary { .. } => {
                self.analyze_binary(expr, env)?;
            }
            Call { ty, args, .. } => {
                res = self.analyze_fn(ty, args, expr, env)?;
            }
            Block { .. } => {
                self.analyze_body(expr, env)?;
            }
            LetStmt {
                pattern,
                init,
                else_block,
            } => {
                self.analyze_let_stmt(pattern, init, else_block, env)?;
            }
            Assign { lhs, rhs } => {
                self.analyze_assign(lhs, rhs, env)?;
            }
            AssignOp { op, lhs, rhs } => {
                self.analyze_assign_op(op, lhs, rhs, env)?;
            }
            Loop { body } => {
                self.analyze_body(body, env)?;
            }
            If {
                cond,
                then,
                else_opt,
            } => {
                self.analyze_if(cond, then, else_opt, env)?;
            }
            Return { value } => match value {
                Some(value) => {
                    let constraint = self.expr_to_const(value.clone(), env)?;
                    env.add_smt_command(constraint.clone(), value.clone());
                    res = AnalysisType::Return(Some(constraint));
                }
                None => res = AnalysisType::Return(None),
            },
            _ => {
                return Err(AnalysisError::Unsupported(
                    format!("Unsupported expression {:?}", expr.kind).to_string(),
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
    Return(Option<String>),
}

#[derive(Debug)]
pub enum AnalysisError {
    Unsupported(String),
    Unimplemented(String),
    FunctionNotFound(LocalDefId),
    VerificationFailed, // { span: Span },
    OutOfBounds(usize),
    RandFunctions,
}
