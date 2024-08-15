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
}
