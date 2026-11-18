//! Points of the legacy plane.

use super::field::Field;
use super::point::{Point, plus};

impl Point for [f64; 2] {
    const SOLID: bool = false;

    fn get(self, field: &impl Field) -> f64 {
        field.flat(self)
    }

    fn mul(self, factor: f64) -> Self {
        [self[0] * factor, self[1] * factor]
    }

    fn div(self, divisor: f64) -> Self {
        [self[0] / divisor, self[1] / divisor]
    }

    fn add(self, shift: [f64; 3]) -> Self {
        [plus(self[0], shift[0]), plus(self[1], shift[1])]
    }

    fn each(value: impl Fn(usize) -> f64) -> Self {
        [value(0), value(1)]
    }

    fn axis(self, k: usize) -> f64 {
        self[k]
    }

    fn onto(self, _origin: Self) -> Self {
        self
    }
}
