use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::LocalDefId;

use std::collections::HashMap as Map;
use std::rc::Rc;

use crate::thir::rthir::RThir;

pub fn get_fn_id_map<'tcx>(tcx: &TyCtxt<'tcx>) -> Map<LocalDefId, Rc<RThir<'tcx>>> {
    let mut fn_map: Map<LocalDefId, Rc<RThir<'tcx>>> = Map::new();
    let fn_keys = tcx.mir_keys(());
    fn_keys.iter().for_each(|&k| {
        let rthir = RThir::generate_rthir(&tcx, k);
        !println!("fn_id: {:?}, rthir: {:?}", k, rthir);
        fn_map.insert(k, Rc::new(rthir));
    });
}
