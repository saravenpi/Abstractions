//! Tuning constants of the ship.

use crate::universe::STANDARD_GRAVITY;

/// Acceleration of the main engine at full throttle in metres per second squared: three times
/// standard gravity, the push the ship had over its weight under the 22 m/s squared it was
/// first tuned for.
pub const MAIN_ACCEL: f64 = 3.0 * STANDARD_GRAVITY;

/// Acceleration of the vertical thrusters in metres per second squared, with the assist off:
/// two and a half times standard gravity.
pub const LIFT_ACCEL: f64 = 2.5 * STANDARD_GRAVITY;

/// Mass of the ship with its crew in kilograms: a small shuttle. The push of its engines on the
/// air under it is this times the acceleration it feels.
pub const SHIP_MASS: f32 = 15000.0;
