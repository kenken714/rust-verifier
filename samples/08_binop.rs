extern crate verify_modules;
use verify_modules::*;

fn main() {
    Vassert(5 + 2 == 7);
    Vassert(5 - 2 == 3);
    Vassert(5 * 2 == 10);
    Vassert(5 / 2 == 2);

    Vassert(-5 % 3 == 1);
    Vassert(5 % 3 == 2);
    Vassert(-5 % -3 == 1);
}
