extern crate verify_modules;
use verify_modules::*;

fn main() {
    let x = 3;
    let y = &mut x;
    *y += 1;
    Vdrop(y);
    Vassert(x == 4);
}
