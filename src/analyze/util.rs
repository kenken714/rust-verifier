use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span,
};

use std::io::Write;
use std::process::Command;

use crate::analyze::*;

impl<'tcx> Analyzer<'tcx> {
    pub fn get_fn(&self, id: LocalDefId) -> Result<Rc<RThir<'tcx>>, AnalysisError> {
        self.fn_map
            .get(&id)
            .cloned()
            .ok_or(AnalysisError::FunctionNotFound(id))
    }

    pub fn get_local_fn(&self, def_id: &DefId) -> Option<Rc<RThir<'tcx>>> {
        if def_id.is_local() {
            Some(
                self.fn_map
                    .get(&def_id.expect_local())
                    .expect("Function not found")
                    .clone(),
            )
        } else {
            None
        }
    }

    pub fn get_fn_info(&self, def_id: &DefId) -> Vec<String> {
        let def_path = self.tcx.def_path_str(*def_id);
        def_path
            .split(|c| c == ':' || c == '"' || c == '\\') //TODO: check
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    pub fn get_name_from_span(span: Span) -> String {
        let mut span_str = format!("{:?}", span);
        span_str = span_str.replace(|c: char| !c.is_alphanumeric(), "_");
        span_str
    }

    pub fn expr_to_var_id(expr: Rc<RExpr<'tcx>>) -> LocalVarId {
        match &expr.kind {
            RExprKind::VarRef { id } => id.clone(),
            RExprKind::Deref { arg } => {
                if let RExprKind::VarRef { id } = arg.kind {
                    id
                } else {
                    panic!("Deref target is not a variable")
                }
            }
            _ => unreachable!(),
        }
    }
}
