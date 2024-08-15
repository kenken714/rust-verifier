extern crate verify_modules;
use verify_modules::*;

fn return_expr(n: i32) -> i32 {
    let m = n + 1;
    m
}

fn main() {
    let x = 5;
    let y = return_expr(x);
    Vassert(y > x);
}
