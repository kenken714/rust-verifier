use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_hir::BindingMode;
use rustc_middle::middle::region;
use rustc_middle::mir::{BinOp, BorrowKind, UnOp};
use rustc_middle::thir::*;
use rustc_middle::ty::adjustment::PointerCoercion;
use rustc_middle::ty::{self, CanonicalUserType, GenericArgsRef, Mutability, Ty};
use rustc_span::{Span, Symbol};
use rustc_target::abi::{FieldIdx, VariantIdx};

use std::fmt;
use std::rc::Rc;

pub struct RThir<'tcx> {
    pub params: Vec<RParam<'tcx>>,
    pub body: Option<Rc<RExpr<'tcx>>>,
}

impl<'tcx> RThir<'tcx> {
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
            body: None,
        }
    }
    pub fn set_params(&mut self, params: Vec<RParam<'tcx>>) {
        self.params = params;
    }
    pub fn set_body(&mut self, body: Option<Rc<RExpr<'tcx>>>) {
        self.body = body;
    }
}

impl<'tcx> fmt::Debug for RThir<'tcx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "params: {:?}, body: {:?}", self.params, self.body)
    }
}

struct RThirFormatter<'tcx> {
    rthir: &'tcx RThir<'tcx>,
}

#[derive(Clone, Debug)]
pub struct RParam<'tcx> {
    pub pat: Option<Rc<RExpr<'tcx>>>,
}

impl<'tcx> RParam<'tcx> {
    pub fn new(pat: Option<Rc<RExpr<'tcx>>>) -> Self {
        Self { pat }
    }
}

#[derive(Clone, Debug)]
pub enum RPatKind<'tcx> {
    Wild,
    AscribeUserType {
        ascription: Ascription<'tcx>,
        subpattern: Rc<RExpr<'tcx>>,
    },
    Binding {
        name: Symbol,
        mode: BindingMode,
        var: LocalVarId,
        ty: Ty<'tcx>,
        subpattern: Option<Rc<RExpr<'tcx>>>, //
        is_primary: bool,
    },
    Deref {
        subpattern: Rc<RExpr<'tcx>>,
    },
    DerefPattern {
        subpattern: Rc<RExpr<'tcx>>,
        mutability: Mutability,
    },
    Range(Box<PatRange<'tcx>>),
    Or {
        pats: Box<[Rc<RExpr<'tcx>>]>,
    },
    Never,
}

#[derive(Clone, Debug)]
pub struct RExpr<'tcx> {
    pub kind: RExprKind<'tcx>,
    pub span: Span,
}

impl<'tcx> RExpr<'tcx> {
    pub fn new(kind: RExprKind<'tcx>, span: Span) -> Self {
        Self { kind, span }
    }
}

type UserTy<'tcx> = Option<Box<CanonicalUserType<'tcx>>>;

#[derive(Clone, Debug)]
pub enum RExprKind<'tcx> {
    If {
        cond: Rc<RExpr<'tcx>>,
        then: Rc<RExpr<'tcx>>,
        else_opt: Option<Rc<RExpr<'tcx>>>,
    },
    Call {
        ty: Ty<'tcx>,
        fun: Rc<RExpr<'tcx>>,
        args: Box<[Rc<RExpr<'tcx>>]>,
        from_hir_call: bool,
        fn_span: Span,
    },
    Deref {
        arg: Rc<RExpr<'tcx>>,
    },
    Binary {
        op: BinOp,
        lhs: Rc<RExpr<'tcx>>,
        rhs: Rc<RExpr<'tcx>>,
    },
    LogicalOp {
        op: LogicalOp,
        lhs: Rc<RExpr<'tcx>>,
        rhs: Rc<RExpr<'tcx>>,
    },
    Unary {
        op: UnOp,
        arg: Rc<RExpr<'tcx>>,
    },
    Cast {
        source: Rc<RExpr<'tcx>>,
    },
    PointerCoercion {
        cast: PointerCoercion,
        source: Rc<RExpr<'tcx>>,
    },
    Loop {
        body: Rc<RExpr<'tcx>>,
    },
    LetBinding {
        expr: Rc<RExpr<'tcx>>,
        pat: Rc<RExpr<'tcx>>,
    },
    Pat {
        kind: RPatKind<'tcx>,
    },
    Match {
        scrutinee: Rc<RExpr<'tcx>>,
        arms: Vec<Rc<RExpr<'tcx>>>,
    },
    Block {
        stmts: Vec<Rc<RExpr<'tcx>>>,
        expr: Option<Rc<RExpr<'tcx>>>,
    },
    Assign {
        lhs: Rc<RExpr<'tcx>>,
        rhs: Rc<RExpr<'tcx>>,
    },
    AssignOp {
        op: BinOp,
        lhs: Rc<RExpr<'tcx>>,
        rhs: Rc<RExpr<'tcx>>,
    },
    Field {
        lhs: Rc<RExpr<'tcx>>,
        variant_index: VariantIdx,
        name: FieldIdx,
    },
    Index {
        lhs: Rc<RExpr<'tcx>>,
        index: Rc<RExpr<'tcx>>,
    },
    VarRef {
        id: LocalVarId,
    },
    UpvarRef {
        closure_def_id: DefId,
        var_hir_id: LocalVarId,
    },
    Borrow {
        borrow_kind: BorrowKind,
        arg: Rc<RExpr<'tcx>>,
    },
    AddressOf {
        mutability: Mutability,
        arg: Rc<RExpr<'tcx>>,
    },
    Break {
        label: region::Scope,
        value: Option<Rc<RExpr<'tcx>>>,
    },
    Continue {
        label: region::Scope,
    },
    Return {
        value: Option<Rc<RExpr<'tcx>>>,
    },
    Repeat {
        value: Rc<RExpr<'tcx>>,
        count: ty::Const<'tcx>,
    },
    Array {
        fields: Box<[Rc<RExpr<'tcx>>]>,
    },
    Tuple {
        fields: Box<[Rc<RExpr<'tcx>>]>,
    },
    PlaceTypeAscription {
        source: Rc<RExpr<'tcx>>,
        user_ty: UserTy<'tcx>,
    },
    ValueTypeAscription {
        source: Rc<RExpr<'tcx>>,
        user_ty: UserTy<'tcx>,
    },
    Literal {
        lit: &'tcx hir::Lit,
        neg: bool,
    },
    NonHirLiteral {
        lit: ty::ScalarInt,
        user_ty: UserTy<'tcx>,
    },
    ZstLiteral {
        user_ty: UserTy<'tcx>,
    },
    NamedConst {
        def_id: DefId,
        args: GenericArgsRef<'tcx>,
        user_ty: UserTy<'tcx>,
    },
    ConstParam {
        param: ty::ParamConst,
        def_id: DefId,
    },
    LetStmt {
        pattern: Rc<RExpr<'tcx>>,
        init: Option<Rc<RExpr<'tcx>>>,
        body: Option<Rc<RExpr<'tcx>>>,
    },
}
