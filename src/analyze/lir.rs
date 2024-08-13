use rustc_middle::ty::{Ty, TyKind};
use rustc_span::Span;

use std::rc::Rc;

use crate::analyze::*;

pub enum LirKind<'tcx> {
    Declaration { name: String, ty: Ty<'tcx> },
    Assume(String),
    Assert(String),
}
pub struct Lir<'tcx> {
    pub kind: LirKind<'tcx>,
    pub expr: Rc<RExpr<'tcx>>,
}

impl<'tcx> Lir<'tcx> {
    pub fn new(kind: LirKind<'tcx>, expr: Rc<RExpr<'tcx>>) -> Self {
        Self { kind, expr }
    }

    pub fn new_param(name: String, ty: Ty<'tcx>, pat: Rc<RExpr<'tcx>>) -> Lir<'tcx> {
        Lir::new(
            LirKind::Declaration {
                name,
                ty: ty.clone(),
            },
            pat.clone(),
        )
    }

    pub fn new_assert(constraint: String, expr: Rc<RExpr<'tcx>>) -> Lir<'tcx> {
        Lir::new(LirKind::Assert(constraint), expr)
    }
    pub fn new_assume(constraint: String, expr: Rc<RExpr<'tcx>>) -> Lir<'tcx> {
        Lir::new(LirKind::Assume(constraint), expr)
    }

    pub fn to_smt(&self) -> Result<String, AnalysisError> {
        use LirKind::*;

        match &self.kind {
            Declaration { name, ty } => match ty.kind() {
                TyKind::Bool => Ok(format!("(declare-const {} Bool\n", name)),
                TyKind::Int(_) => Ok(format!("(declare-const {} Int\n", name)),
                _ => Err(AnalysisError::Unsupported(
                    "Only Int and Bool types are supported".to_string(),
                )),
            },
            Assert(constraint) => Ok(format!("(assert (not {}))", constraint)),
            Assume(constraint) => Ok(format!("(assert ({}))", constraint)),
            _ => Err(AnalysisError::Unsupported(
                "Unsupported annotation kind".to_string(),
            )),
        }
    }

    pub fn to_assert(&self) -> Result<String, AnalysisError> {
        use LirKind::*;
        match &self.kind {
            Assert(constraint) => Ok(format!("(assert (not {}))", constraint)),
            Assume(constraint) => Ok(format!("(assert (not {}))", constraint)),
            _ => Err(AnalysisError::Unsupported(
                "Unsupported annotation kind".to_string(),
            )),
        }
    }
}
