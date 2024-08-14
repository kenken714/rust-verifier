pub fn Vassert(_: bool) {}
pub fn Vassume(_: bool) {}
pub fn Vinvariant(_: bool) {}
pub fn Vrand_int<T: From<i32>>() -> T {
    T::from(0)
}
pub fn Vrand_bool<T: From<bool>>() -> T {
    T::from(false)
}
pub fn Vrand_float<T: From<f64>>() -> T {
    T::from(0.0)
}
pub fn Vdrop<T>(_: T) {}
