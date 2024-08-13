use rustc_hir::Lit;
use rustc_middle::mir::{BinOp, UnOp};
use rustc_middle::thir::LocalVarId;
use rustc_middle::thir::LogicalOp;
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::{Ty, TyKind};
use rustc_span::def_id::LocalDefId;

use std::boxed::Box;
use std::collections::HashMap as Map;
use std::iter::Peekable;
use std::rc::Rc;

use crate::thir::rthir::*;
pub mod core;
mod env;
mod expr;
mod lir;
mod util;

pub use {core::AnalysisError, env::Env, expr::*, lir::*};

pub fn analyze<'tcx>(
    main_id: LocalDefId,
    fn_map: Map<LocalDefId, Rc<RThir<'tcx>>>,
    tcx: TyCtxt<'tcx>,
) -> Result<(), AnalysisError> {
    Analyzer::run(main_id, fn_map, tcx)
}

struct Analyzer<'tcx> {
    fn_map: Map<LocalDefId, Rc<RThir<'tcx>>>,
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> Analyzer<'tcx> {
    pub fn new(fn_map: Map<LocalDefId, Rc<RThir<'tcx>>>, tcx: TyCtxt<'tcx>) -> Self {
        Self { fn_map, tcx }
    }

    pub fn run(
        main_id: LocalDefId,
        fn_map: Map<LocalDefId, Rc<RThir<'tcx>>>,
        tcx: TyCtxt<'tcx>,
    ) -> Result<(), AnalysisError> {
        let analyzer = Analyzer::new(fn_map, tcx);
        let main = analyzer.fn_map(main_id)?;
        analyzer.analyze_enter(main)
    }

    //unimplemented
}
