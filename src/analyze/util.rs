use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span,
};

use std::io::Write;
use std::process::Command;

use crate::analyze::*;

impl<'tcx> Analyzer<'tcx> {
    pub fn verify_z3(&self, mut smt_str: String, env: &mut Env<'tcx>) -> Result<(), AnalysisError> {
        let mut command = Command::new("z3")
            .arg("-in")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn z3 process");
        smt_str.push_str("\n(check-sat)\n");

        let stdin = command.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(smt_str.as_bytes())
            .expect("Failed to write to stdin");
        drop(stdin);

        let output = command.wait_with_output().expect("Failed to read stdout");
        let output_str = String::from_utf8(output.stdout).expect("Failed to convert to string");

        println!("SMT: \n {}", smt_str);
        println!("Output: \n {}", output_str);
        if output_str.contains("unsat") {
            Ok(())
        } else {
            Err(AnalysisError::VerificationFailed)
        }
    }
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
}
