use rustc_middle::ty::{Ty, TyKind};
use rustc_span::Span;

use std::rc::Rc;

use crate::analyze::*;

#[derive(Clone)]
pub enum LirKind<'tcx> {
    Declaration { name: String, ty: Ty<'tcx> },
    Assume(String),
    Assert(String),
}

#[derive(Clone)]
pub struct Lir<'tcx> {
    pub kind: LirKind<'tcx>,
    pub expr: Rc<RExpr<'tcx>>,
    pub assume: Option<String>,
}

impl<'tcx> Lir<'tcx> {
    pub fn new(kind: LirKind<'tcx>, expr: Rc<RExpr<'tcx>>, assume: Option<String>) -> Self {
        Self { kind, expr, assume }
    }

    pub fn new_param(
        name: String,
        ty: Ty<'tcx>,
        pat: Rc<RExpr<'tcx>>,
        assume: Option<String>,
    ) -> Lir<'tcx> {
        Lir::new(
            LirKind::Declaration {
                name,
                ty: ty.clone(),
            },
            pat.clone(),
            assume.clone(),
        )
    }

    pub fn new_assert(
        constraint: String,
        expr: Rc<RExpr<'tcx>>,
        assume: Option<String>,
    ) -> Lir<'tcx> {
        Lir::new(LirKind::Assert(constraint), expr, assume)
    }
    pub fn new_assume(
        constraint: String,
        expr: Rc<RExpr<'tcx>>,
        assume: Option<String>,
    ) -> Lir<'tcx> {
        Lir::new(LirKind::Assume(constraint), expr, assume)
    }

    pub fn set_assume(&mut self, constraint: Option<String>) {
        self.assume = constraint;
    }

    pub fn adopt_assume(&mut self, op: String, arg: String, expr: Rc<RExpr<'tcx>>) {
        self.assume = Some(format!(
            "({} {} {})",
            op,
            self.assume.clone().expect("assume not exist"),
            arg
        ));
        self.expr = expr;
    }
}
