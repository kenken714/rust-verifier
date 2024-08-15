extern crate verify_modules;
use verify_modules::*;
fn main() {
    let x = Vrand_int::<i32>();
    let y = if x >= 0 {
        let n = x + 5;
        Vassert(n >= 0);
        n
    } else {
        let n = x - 5;
        Vassert(n < 0);
        n * -1
    };
    Vassert(y >= 0);
}
