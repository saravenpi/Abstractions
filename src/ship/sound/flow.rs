//! How the air meets the ship this machine flies, for the sound of its airframe: the angle of
//! attack and the sideslip of the flight path against the nose, how fast it rolls and yaws, and
//! the load of a turn. Read from the real velocity and spin of the hull.

use super::super::body::Ship;
use crate::audio::{Ch, HullSound};
use crate::universe::STANDARD_GRAVITY;

/// Speed in metres per second under which the flight path has no direction to read angles from.
const STILL: f64 = 1.0;

/// Returns `hull` with the flow of `ship` on it. The body axes follow Bevy: the nose along -Z,
/// the roof along +Y, the right wing along +X. A pull of the stick turns the path at the pitch
/// rate, so the load is one g plus the pitch rate times the speed over gravity.
pub fn flow(hull: HullSound, ship: &Ship) -> HullSound {
    let body = ship.rot.inverse() * ship.vel;
    let speed = body.length();
    let (attack, slip) = match speed > STILL {
        true => ((-body.y).atan2(-body.z).abs(), body.x.atan2(-body.z)),
        false => (0.0, 0.0),
    };
    let turn = ship.spin.y.hypot(ship.spin.z);
    let load = 1.0 + ship.spin.x.abs() * speed / STANDARD_GRAVITY;
    hull.with(Ch::Attack, attack.min(1.6) as f32)
        .with(Ch::Slip, slip.clamp(-1.6, 1.6) as f32)
        .with(Ch::Turn, turn as f32)
        .with(Ch::Load, load as f32)
}
