use rustc_middle::ty::{Ty, TyKind};
use rustc_span::Span;

use std::rc::Rc;

pub enum LirKind<'tcx> {
    Declaration { name: String, ty: Ty<'tcx> },
    Assume(String),
    Assert(String),
    Assumptions(String),
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
    pub fn new_assumption(constraint: String, expr: Rc<RExpr<'tcx>>) -> Lir<'tcx> {
        Lir::new(LirKind::Assumption(constraint), expr)
    }

    pub fn to_smt(&self) -> Result<String, ()> {
        use LirKind::*;

        match &self.kind {
            Declaration { name, ty } => match ty.kind() {
                TyKind::Bool => Ok(format!("(declare-const {} Bool\n", name)),
                TyKind::Int => Ok(format!("(declare-const {} Int\n", name)),
            },
            Assert(constraint) => Ok(format!("(assert (not {}))", name)),
            Assume(constraint) => Ok(format!("(assert ({}))", name)),
            _ => Err(),
        }
    }
}
