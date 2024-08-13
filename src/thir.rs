use rustc_middle::ty::TyCtxt;
use rustc_span::{def_id::LocalDefId, ErrorGuaranteed};

mod printer;
mod reduce;
pub mod rthir;

use reduce::*;
use rthir::RThir;

pub fn generate_rthir<'tcx>(
    tcx: &TyCtxt<'tcx>,
    owner_def: LocalDefId,
) -> Result<RThir<'tcx>, ErrorGuaranteed> {
    let (thir, _) = tcx.thir_body(owner_def)?;
    let thir = thir.steal();
    Ok(reduce::reduce_thir(thir))
}
