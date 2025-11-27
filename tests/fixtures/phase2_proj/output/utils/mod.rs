pub mod math;

// Content from index.ts
use self::math::add;
pub fn add_and_double(a: f64, b: f64) -> f64 {
    return add(a, b) * 2f64;
}
