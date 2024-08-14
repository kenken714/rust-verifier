extern crate verify_modules;
use verify_modules::*;
fn f(n: i32) {
    let m = n + 1;
    Vassert(m > 4);
}

fn main() {
    let x = 4;
    f(x);
}
