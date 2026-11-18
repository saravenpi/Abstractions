//! Sample points: the one place where the flat plane and the sphere differ when noise is read.

use super::field::Field;

/// A position, in metres, at which the noise fields are read.
///
/// `[f64; 2]` is a point of the legacy plane. `[f64; 3]` is a point of a sphere around the planet
/// centre. Every ground formula is written once over this trait, so both read the same world rules.
pub trait Point: Copy {
    /// True for a point of a sphere: the rules only the globe follows, such as the relief its gravity
    /// allows, read this so that the legacy plane never changes.
    const SOLID: bool;

    /// Returns the value of `field` at this point.
    fn get(self, field: &impl Field) -> f64;

    /// Returns the point with every coordinate multiplied by `factor`.
    fn mul(self, factor: f64) -> Self;

    /// Returns the point with every coordinate divided by `divisor`.
    fn div(self, divisor: f64) -> Self;

    /// Returns the point moved by the leading coordinates of `shift`. A zero leaves its axis untouched.
    fn add(self, shift: [f64; 3]) -> Self;

    /// Builds a point from `value(k)` for each axis `k`.
    fn each(value: impl Fn(usize) -> f64) -> Self;

    /// Returns coordinate `k`.
    fn axis(self, k: usize) -> f64;

    /// Returns the point brought back onto the surface that holds `origin`.
    ///
    /// The plane holds every point. A sphere point is scaled to the distance of `origin` from the centre.
    fn onto(self, origin: Self) -> Self;
}

/// Returns `a + shift`, or `a` itself when `shift` is zero, so a zero shift keeps even the sign of zero.
pub fn plus<T: PartialEq + Default + std::ops::Add<Output = T>>(a: T, shift: T) -> T {
    if shift == T::default() { a } else { a + shift }
}
