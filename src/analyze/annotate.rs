use crate::analyze::core::AnalysisType;
use crate::analyze::*;

impl<'tcx> Analyzer<'tcx> {
    pub fn analyze_assert(
        &self,
        args: Box<[Rc<RExpr<'tcx>>]>,
        env: &mut Env<'tcx>,
    ) -> Result<AnalysisType<'tcx>, AnalysisError> {
        self.analyze_assume(args, env)?;
        let smt = env.get_smt_command_for_assume()?;
        self.verify_z3(smt, env)?;
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
}
