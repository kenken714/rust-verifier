use rustc_middle::ty::TyCtxt;

use crate::analyze::core::*;

use crate::analyze::analyze;
use crate::util::get_fn_id_map;
use std::fs::File;
use std::io::Result;
use std::path::PathBuf;

pub struct Options {
    pub output_file: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        // not impremented
        Options {
            output_file: PathBuf::from("output.smt2"),
        }
    }
}
pub fn drive_rust_verifier(tcx: TyCtxt, opts: &Options) -> Result<()> {
    if let Some((entry_def_id, _)) = tcx.entry_fn(()) {
        let mut output_file = File::create(&opts.output_file).unwrap();
        let fn_id_map = get_fn_id_map(&tcx);
        println!("Entry function found: {:?}", entry_def_id);
        //output tcx
        if let Err(error) = analyze(entry_def_id.expect_local(), fn_id_map, tcx) {
            use AnalysisError::*;
            match error {
                Unsupported(message) => {
                    println!("Unsupported: {}", message);
                }
                Unimplemented(message) => {
                    println!("Unimplemented: {}", message);
                }
                FunctionNotFound(id) => {
                    println!("Function not found: {:?}", id);
                }
                VerificationFailed => {
                    println!("Verification failed");
                }
                OutOfBounds(idx) => {
                    println!("Out of bounds: {}", idx);
                }
                RandFunctions => {
                    println!("Rand functions");
                }
            }
        }
    } else {
        panic!("No main function found");
    }
    Ok(())
}
