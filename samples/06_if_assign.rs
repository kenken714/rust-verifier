extern crate verify_modules;
use verify_modules::*;

fn f(n: i32) -> i32 {
    let m = n + 1;
    m
}

fn main() {
    let x = Vrand_int();
    let y = f(x);
    Vassert(y > x);
}
