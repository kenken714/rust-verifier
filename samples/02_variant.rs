extern crate verify_modules;
use verify_modules::*;
fn main() {
    let mut x = 1;
    Vassert(x <= 2);
    x = 0;
    Vassert(x == 0);
    x += 1;
    Vassert(x == 1);
}
