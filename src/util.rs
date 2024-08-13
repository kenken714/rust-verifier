use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::LocalDefId;

use std::collections::HashMap;
use std::rc::Rc;

use crate::thir::generate_rthir;
use crate::thir::rthir::RThir;

pub fn get_fn_id_map<'tcx>(tcx: &TyCtxt<'tcx>) -> HashMap<LocalDefId, Rc<RThir<'tcx>>> {
    let mut fn_map: HashMap<LocalDefId, Rc<RThir<'tcx>>> = HashMap::new();
    let fn_keys = tcx.mir_keys(());
    fn_keys.iter().for_each(|&k| {
        let rthir = generate_rthir(&tcx, k).expect("Failed to generate rthir");
        println!("fn_id: {:?}, rthir: {:?}", k, rthir);
        fn_map.insert(k, Rc::new(rthir));
    });
    fn_map
}
