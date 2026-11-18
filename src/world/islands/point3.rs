//! Points of a sphere around the planet centre.

use super::field::Field;
use super::point::{Point, plus};

impl Point for [f64; 3] {
    const SOLID: bool = true;

    fn get(self, field: &impl Field) -> f64 {
        field.solid(self)
    }

    fn mul(self, factor: f64) -> Self {
        self.map(|c| c * factor)
    }

    fn div(self, divisor: f64) -> Self {
        self.map(|c| c / divisor)
    }

    fn add(self, shift: [f64; 3]) -> Self {
        [0, 1, 2].map(|k| plus(self[k], shift[k]))
    }

    fn each(value: impl Fn(usize) -> f64) -> Self {
        [value(0), value(1), value(2)]
    }

    fn axis(self, k: usize) -> f64 {
        self[k]
    }

    fn onto(self, origin: Self) -> Self {
        let length = |p: Self| p.iter().map(|c| c * c).sum::<f64>().sqrt();
        self.mul(length(origin) / length(self))
    }
}
