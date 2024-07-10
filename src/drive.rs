use rustc_middle::ty::TyCtxt;

use std::fs::File;
use std::io::Result;
use std::path::PathBuf;

//use crate::analyze::analyze;
//use crate::util::get_fn_id_map;

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
        //let fn_id_map = get_fn_id_map(&tcx);
        println!("Entry function found: {:?}", entry_def_id);
        //output tcx
        //analyze(entry_def_id.expect_local(), &fn_id_map)?;
    } else {
        panic!("No main function found");
    }
    Ok(())
}
