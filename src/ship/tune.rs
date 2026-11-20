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

/// Metres above sea level of the orbit the climb of the autopilot flies to, when the air of the
/// planet ends below it.
pub const ORBIT_ALTITUDE: f64 = 200_000.0;

/// Metres the apoapsis and the periapsis of the orbit may be off the height asked for before the
/// climb counts as done.
pub const ORBIT_TOLERANCE: f64 = 10_000.0;

/// Seconds in which the climb closes the gap between the ship and the height of its orbit: the
/// climb speed is the gap over this time.
pub const CLIMB_TIME: f64 = 4.0;

/// Share of the height of the orbit at which the climb has the whole sideways speed of a
/// circular orbit. Under it the sideways speed grows with the square of the height.
pub const ORBIT_RAMP: f64 = 0.8;

/// Times the speed of a circular orbit at the local radius that the speed hold may fly in open
/// space, where that is more than its fixed top speed of 2,500 metres per second: a ship
/// climbing from a planet has to reach the speed of its orbit, and escapes it at 1.41 times that.
pub const ORBIT_CAP: f64 = 2.0;

/// Radii of the Hill sphere of a body between which it stops carrying the warp bubble: it
/// carries it whole inside the first and not at all outside the second.
pub const CARRY: (f64, f64) = (0.25, 0.5);

/// Share of the Hill radius of a planet or a moon inside which it is the body the world streams.
pub const BODY_SWITCH: f64 = 0.1;

/// Share of the Hill radius of a planet inside which it carries a ship across a gap in time.
pub const ESCORT: f64 = 0.5;
