use rustc_middle::{
    mir::coverage::Op,
    ty::{self, Ty, TyKind},
};
use rustc_span::{sym::TyKind, Span};

use std::rc::Rc;

use crate::analyze::*;

#[derive(Clone)]
pub enum LirKind<'tcx> {
    //TODO:消す
    Declaration { name: String, ty: Ty<'tcx> },
    Assume(String),
}

#[derive(Debug, Clone)]
pub enum LirDatKind<'tcx> {
    Path {
        assume: String,
    },
    Aggregate {
        fields: Vec<LirDatKind<'tcx>>,
        ty: Ty<'tcx>,
    },
}

#[derive(Clone)]
pub struct Lir<'tcx> {
    pub kind: LirKind<'tcx>,
    pub expr: Rc<RExpr<'tcx>>,

    pub kind_dat: Option<LirDatKind<'tcx>>,
}

impl<'tcx> Lir<'tcx> {
    pub fn new(
        kind: LirKind<'tcx>,
        ty: Option<Ty<'tcx>>,
        expr: Rc<RExpr<'tcx>>,
        assume: Vec<Option<String>>,
    ) -> Self {
        let kind_dat = if let Some(ty) = ty {
            let kind_dat = if let TyKind::Ref(_, ty, _) = ty.kind() {
                LirDatKind::new_aggregate(vec![ty.to_string()], ty.clone())
            } else {
                LirDatKind::new(assume[0].clone())
            };
            kind_dat
        } else {
            LirDatKind::new(assume[0].clone())
        };

        Self {
            kind,
            expr,
            kind_dat: Some(kind_dat),
        }
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
            Some(ty),
            pat.clone(),
            vec![assume],
        )
    }

    pub fn new_assume(
        constraint: String,
        ty: Option<Ty<'tcx>>,
        expr: Rc<RExpr<'tcx>>,
        assume: Option<String>,
    ) -> Lir<'tcx> {
        Lir::new(LirKind::Assume(constraint), ty, expr, vec![assume])
    }

    pub fn set_assume(&mut self, constraint: Option<String>) {
        match &mut self.kind_dat {
            Some(LirDatKind::Path { assume, .. }) => {
                *assume = constraint.expect("assume not exist");
            }
            _ => panic!("set_assume failed; not path"),
        }
    }

    pub fn adopt_assume(&mut self, op: String, arg: String, expr: Rc<RExpr<'tcx>>) {
        match &mut self.kind {
            LirKind::Assume(constraint) => {
                *constraint = format!("({} {} {})", op, constraint.clone(), arg);
            }
            _ => panic!("adopt_assume failed; not assume"),
        }
        self.expr = expr;
    }

    pub fn get_assume(&self) -> String {
        match &self.kind_dat {
            Some(dat) => dat.get_assume(),
            _ => panic!("get_assume failed; assume not exist"),
        }
    }

    pub fn get_assume_by_idx(&self, idx: Vec<usize>) -> String {
        match &self.kind_dat {
            Some(dat) => dat.get_assume_by_idx(idx),
            _ => panic!("get_assume failed; assume not exist"),
        }
    }
}

impl<'tcx> LirDatKind<'tcx> {
    //ここから下を頑張って実装するよ

    pub fn new(assume: Option<String>) -> Self {
        Self::Path {
            assume: assume.expect("assume not exist when new LirDatKind::Path"),
        }
    }

    pub fn new_aggregate(arg: Vec<String>, ty: Ty<'tcx>) -> Self {
        Self::Aggregate {
            fields: arg
                .iter()
                .map(|arg| Self::new(Some(arg.to_string())))
                .collect(),
            ty,
        }
    }
    pub fn get_assume(&self) -> String {
        match &self {
            LirDatKind::Path { assume } => assume.clone(),
            LirDatKind::Aggregate { fields, .. } => fields[0].get_assume(),
            _ => panic!("get_assume failed; not path"),
        }
    }

    pub fn get_assume_by_idx(&self, mut idx: Vec<usize>) -> String {
        match &self {
            LirDatKind::Path { assume } => assume.clone(),
            LirDatKind::Aggregate { fields, .. } => fields[idx.remove(0)].get_assume_by_idx(idx),
            _ => panic!("get_assume failed; not path"),
        }
    }

    pub fn set_assume(&mut self, constraint: Option<String>) {
        match self {
            LirDatKind::Path { assume } => {
                *assume = constraint.expect("assume not exist");
            }
            LirDatKind::Aggregate { fields, .. } => {
                fields[0].set_assume(constraint);
            }
        }
    }
}
