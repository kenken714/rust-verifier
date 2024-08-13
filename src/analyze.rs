use rustc_ast::ast::LitKind;
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

pub use {env::Env, helper_struct::*, lir::*};

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
        let main = analyzer.get_fn_id_map(main_id)?;
        analyzer.analyze(main)
    }

    fn analyze_loop(
        &self,
        invariant: Rc<RExpr<'tcx>>,
        stmts_iter: &mut Peekable<impl Iterator<Item = Rc<RExpr<'tcx>>>>,
        env: &mut Env<'tcx>,
    ) -> Result<(), AnalysisError> {
        // verify before loop
        let mut invariants = vec![invariant.clone()];
        let mut before_loop_env = env.gen_new_env("loop".to_string(), invariant.clone())?;
        self.verify_before_loop(
            invariant.clone(),
            &mut invariants,
            stmts_iter,
            &mut before_loop_env,
        )?;

        let expr = stmts_iter.next().expect("No loop expression!");
        if let RExprKind::Loop { body } = expr.kind.clone() {
            let mut break_envs = Vec::new();
            let mut loop_env =
                before_loop_env.gen_new_env("inner_loop".to_string(), expr.clone())?;
            self.verify_loop_internals(body.clone(), invariants, &mut break_envs, &mut loop_env)?;
        } else {
            return Err(AnalysisError::UnsupportedPattern(
                "Multiple invariant is not suppoerted".into(),
            ));
        }

        Ok(())
    }

    fn verify_before_loop(
        &self,
        invariant: Rc<RExpr<'tcx>>,
        invariants: &mut Vec<Rc<RExpr<'tcx>>>,
        stmts_iter: &mut Peekable<impl Iterator<Item = Rc<RExpr<'tcx>>>>,
        env: &mut Env<'tcx>,
    ) -> Result<(), AnalysisError> {
        let constraint = self.expr_to_constraint(invariant.clone(), env)?;
        env.add_assumption(constraint.clone(), invariant.clone());
        let assumptions = env.get_assumptions_for_verify()?;
        self.verify(assumptions, env)?;
        while let Some(inv) = stmts_iter.next_if(|stmt| self.is_invariant(stmt.clone())) {
            if let RExprKind::Call { args, .. } = &inv.kind {
                invariants.push(args[0].clone());
                let constraint = self.expr_to_constraint(args[0].clone(), env)?;
                env.add_assumption(constraint, inv);
                let assumptions = env.get_assumptions_for_verify()?;
                self.verify(assumptions, env)?;
            }
        }
        Ok(())
    }

    fn is_invariant(&self, expr: Rc<RExpr<'tcx>>) -> bool {
        match &expr.kind {
            RExprKind::Call { ty, .. } => match ty.kind() {
                TyKind::FnDef(def_id, ..) => {
                    let fn_info = self.get_fn_info(def_id);
                    match fn_info[1].as_str() {
                        "invariant" => true,
                        _ => false,
                    }
                }
                _ => panic!("Call has not have FnDef"),
            },
            _ => false,
        }
    }

    fn verify_loop_internals(
        &self,
        block: Rc<RExpr<'tcx>>,
        invariants: Vec<Rc<RExpr<'tcx>>>,
        _break_envs: &mut Vec<Env<'tcx>>,
        env: &mut Env<'tcx>,
    ) -> Result<(), AnalysisError> {
        self.set_var_map(block.clone(), invariants, env);
        self.analyze_block(block, env)?;
        let smt = env.get_assumptions_for_verify()?;
        println!("{}", smt);
        self.verify(smt, env)
    }

    fn set_var_map(
        &self,
        block: Rc<RExpr<'tcx>>,
        invariants: Vec<Rc<RExpr<'tcx>>>,
        env: &mut Env<'tcx>,
    ) {
        let inv_varv = Analyzer::search_inv(invariants);
        let varv = Analyzer::search_used_var(block.clone());
        println!("{:?}", varv);
        let refresh_varv = varv.iter().filter(|var| !inv_varv.contains(var));
        for var in refresh_varv {
            let (current_name, ty) = env.var_map.get(var).unwrap().clone();
            let new_name = format!("{}_{}", env.name, current_name);
            env.add_parameter(new_name.clone(), &ty, var, block.clone());
            env.var_map.insert(*var, (new_name, ty.clone()));
        }
    }

    fn search_inv(invariants: Vec<Rc<RExpr<'tcx>>>) -> Vec<LocalVarId> {
        let mut varv: Vec<LocalVarId> = Vec::new();

        for invariant in invariants {
            Analyzer::search_var(invariant, &mut varv);
        }
        varv
    }

    fn search_var(expr: Rc<RExpr<'tcx>>, varv: &mut Vec<LocalVarId>) {
        use RExprKind::*;

        match expr.kind.clone() {
            VarRef { id } => {
                varv.push(id.clone());
            }
            LogicalOp { lhs, rhs, .. } => {
                Analyzer::search_var(lhs.clone(), varv);
                Analyzer::search_var(rhs.clone(), varv);
            }
            Unary { arg, .. } => {
                Analyzer::search_var(arg.clone(), varv);
            }
            Binary { lhs, rhs, .. } => {
                Analyzer::search_var(lhs.clone(), varv);
                Analyzer::search_var(rhs.clone(), varv);
            }
            _ => {
                println!("{:?}", expr.kind);
                panic!("Unknown invariant pattern")
            }
        }
    }

    fn search_used_var(block: Rc<RExpr<'tcx>>) -> Vec<LocalVarId> {
        let mut varv: Vec<LocalVarId> = Vec::new();
        if let RExpr {
            kind: RExprKind::Block { stmts, expr },
            ..
        } = block.as_ref()
        {
            for stmt in stmts {
                Analyzer::search_var_expr(stmt.clone(), &mut varv, false);
            }
            if let Some(expr) = expr {
                Analyzer::search_var_expr(expr.clone(), &mut varv, false);
            }
        }
        varv
    }

    fn search_var_expr(expr: Rc<RExpr<'tcx>>, varv: &mut Vec<LocalVarId>, is_assign: bool) {
        use RExprKind::*;

        match &expr.kind {
            Literal { .. } => (),
            VarRef { id } => {
                if is_assign {
                    varv.push(id.clone());
                }
            }
            LogicalOp { lhs, rhs, .. } => {
                Analyzer::search_var_expr(lhs.clone(), varv, is_assign);
                Analyzer::search_var_expr(rhs.clone(), varv, is_assign);
            }
            Unary { arg, .. } => {
                Analyzer::search_var_expr(arg.clone(), varv, is_assign);
            }
            Binary { lhs, rhs, .. } => {
                Analyzer::search_var_expr(lhs.clone(), varv, is_assign);
                Analyzer::search_var_expr(rhs.clone(), varv, is_assign);
            }
            Call { .. } => (),
            If { then, else_opt, .. } => {
                Analyzer::search_var_expr(then.clone(), varv, is_assign);
                if let Some(else_block) = else_opt {
                    Analyzer::search_var_expr(else_block.clone(), varv, is_assign);
                }
            }
            LetStmt { initializer, .. } => {
                if let Some(initializer) = initializer {
                    Analyzer::search_var_expr(initializer.clone(), varv, is_assign);
                }
            }
            AssignOp { lhs, rhs, .. } => {
                Analyzer::search_var_expr(lhs.clone(), varv, true);
                Analyzer::search_var_expr(rhs.clone(), varv, false);
            }
            Assign { lhs, rhs } => {
                Analyzer::search_var_expr(lhs.clone(), varv, true);
                Analyzer::search_var_expr(rhs.clone(), varv, false);
            }
            Block { stmts, expr } => {
                for stmt in stmts {
                    Analyzer::search_var_expr(stmt.clone(), varv, is_assign);
                }
                if let Some(expr) = expr {
                    Analyzer::search_var_expr(expr.clone(), varv, false);
                }
            }
            Break { .. } => (),
            _ => panic!("Unknown pattern in loop: {:?}", expr),
        }
    }
}
