extern crate verify_modules;
use verify_modules::*;

fn main() {
    let x = Vrand_int::<i32>();
    let y = x + 1;
    Vassert(y != x);
}
