extern crate verify_modules;
use verify_modules::*;

fn main() {
    let mut x = 1;
    if x == 1 {
        x += 1;
    }
    Vassert(x == 2);
}
