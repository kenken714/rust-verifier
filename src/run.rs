use rustc_driver::{Callbacks, Compilation, RunCompiler};
use rustc_interface::{
    interface::{Compiler, Config},
    Queries,
};
use rustc_session::config::OptLevel;
use std::path::PathBuf;

use crate::drive::{drive_rust_verifier, Options};

struct MyCallbacks {
    opts: Options,
}

impl Callbacks for MyCallbacks {
    fn config(&mut self, config: &mut Config) {
        let opts = &mut config.opts;
        opts.optimize = OptLevel::Aggressive;
        opts.debug_assertions = false;
    }

    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        queries.global_ctxt().unwrap().enter(|tcx| {
            let res = drive_rust_verifier(tcx, &self.opts);
            res.unwrap();
        });
        Compilation::Stop
    }
}

pub fn run_rust_verifier() {
    println!("Running rust-analyzer");
    let mut args = Vec::new();
    let mut args_iter = std::env::args();
    let mut opts = Options::default();
    while let Some(arg) = args_iter.next() {
        if arg == "-o" {
            opts.output_file = PathBuf::from(args_iter.next().unwrap());
        } else {
            args.push(arg);
        }
    }
    RunCompiler::new(&args, &mut MyCallbacks { opts })
        .run()
        .unwrap();
}
