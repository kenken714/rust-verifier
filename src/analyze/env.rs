use rustc_middle::thir::LocalVarId;
use rustc_middle::ty::Ty;
use rustc_span::Span;
use std::rc::Rc;

use std::collections::{HashMap, VecDeque};

use crate::analyze::core::AnalysisError;
use crate::analyze::lir::Lir;
use crate::analyze::LirKind;

use super::RExpr;

pub struct Env<'tcx> {
    pub name: String,
    pub path: VecDeque<Lir<'tcx>>,
    pub env_map: HashMap<LocalVarId, Ty<'tcx>>,
}

impl<'tcx> Env<'tcx> {
    pub fn new() -> Self {
        Self {
            name: String::from("main"),
            path: VecDeque::new(),
            env_map: HashMap::new(),
        }
    }

    pub fn from(
        name: String,
        path: VecDeque<Lir<'tcx>>,
        env_map: HashMap<LocalVarId, Ty<'tcx>>,
    ) -> Self {
        Self {
            name,
            path,
            env_map,
        }
    }

    pub fn add_smt_command(&mut self, constraint: String, expr: Rc<RExpr<'tcx>>) {
        self.path.push_back(Lir::new_assert(constraint, expr));
    }

    pub fn get_smt_commands(&self) -> Result<String, AnalysisError> {
        let smt_str = self
            .path
            .iter()
            .map(|smt_command| smt_command.to_smt().unwrap())
            .collect::<Vec<String>>();
        Ok(smt_str.join("\n"))
    }

    pub fn get_smt_command(&self, idx: usize) -> Result<String, AnalysisError> {
        self.path
            .get(idx)
            .ok_or(AnalysisError::OutOfBounds(idx))
            .and_then(|smt_command| Ok(smt_command.to_smt()?))
    }

    //TODO: fix 仮で書いている
    pub fn get_smt_command_for_assume(&self) -> Result<String, AnalysisError> {
        let len = self.path.len();
        let mut command = String::new();
        for i in 0..(len - 1) {
            if let LirKind::Assume(_) = self.path[i].kind {
                command.push_str(&self.path[i].to_smt()?);
            }
        }
        command.push_str(&self.path[len - 1].to_assert()?);
        Ok(command)
    }
}
