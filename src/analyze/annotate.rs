use crate::analyze::core::AnalysisType;
use crate::analyze::*;

impl<'tcx> Analyzer<'tcx> {
    pub fn analyze_assert(
        &self,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        let constraint = self.expr_to_const(args[0].clone(), env)?;
        env.verify_z3(constraint, args[0].span)?;
        Ok(AnalysisType::Other)
    }

    pub fn analyze_assume(
        &self,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        let constraint = self.expr_to_const(args[0].clone(), env)?;
        env.add_smt_command(constraint, args[0].clone());
        Ok(AnalysisType::Other)
    }

    pub fn analyze_invariant(
        &self,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        let name = self.expr_to_const(args[0].clone(), env)?;
        let ty = self.expr_to_const(args[1].clone(), env)?;
        env.add_smt_command(format!("(declare-const {} {})", name, ty), args[0].clone());
        Ok(AnalysisType::Other)
    }

    pub fn analyze_drop(
        &self,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        if let RExprKind::VarRef { id } = &args[0].kind {
            let var = env
                .env_map
                .get(id)
                .expect("Drop failed; target variable not found");
            env.add_smt_command(
                format!(
                    "(= {} {})",
                    var.get_assume(),
                    var.get_assume_by_idx(vec![1])
                ),
                args[0].clone(),
            );
            Ok(AnalysisType::Other)
        } else {
            Err(AnalysisError::Unsupported(
                "Drop target is not a variable".to_string(),
            ))
        }
    }
}
