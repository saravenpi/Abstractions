//! How far the light of a lamp reaches, from its power and the inverse square law.

use std::f32::consts::PI;

use super::tune::LOCAL_FLOOR_LUX;

/// Returns the distance in metres at which a lamp of `lumens` lights a surface facing it with
/// [`LOCAL_FLOOR_LUX`]: the luminous intensity is `lumens / 4 pi` candela, and the illuminance
/// at a distance `d` is that over `d^2`. Past it the lamp is lost in the ambient light of the
/// night, so it is the range to give the light. A lamp of no power has no reach.
pub fn reach(lumens: f32) -> f32 {
    (lumens.max(0.0) / (4.0 * PI * LOCAL_FLOOR_LUX)).sqrt()
}
