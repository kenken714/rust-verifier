extern crate verify_modules;
use verify_modules::*;
fn main() {
    Vassume(1 <= 2);
    Vassert(3 <= 7);
}
