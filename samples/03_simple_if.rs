extern crate verify_modules;
use verify_modules::*;

fn main() {
    let mut x = 1;
    x = if x == 1 { 2 } else { 3 };
    Vassert(x == 2);
}
