extern crate verify_modules;
use verify_modules::*;
fn main() {
    let x = Vrand_int::<i32>();
    if x >= 0 {
        x += 5;
    } else {
        x *= -1;
    }
    Vassert(x >= 0);
}
