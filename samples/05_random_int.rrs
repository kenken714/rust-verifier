extern crate verify_modules;
use verify_modules::*;

fn f(n: i32) {
    let m = n * n;
    Vassert(m >= 9);
}
fn main() {
    let mut x = Vrand_int();
    Vassume(x >= 3);
    f(x);
}
