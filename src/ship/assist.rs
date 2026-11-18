//! Flight assist: turns stick input into rates, kills rotation and levels a hovering ship.
//! The speed it holds is in `cruise`.

use bevy::math::{DQuat, DVec3};

/// Angular acceleration of the attitude thrusters around pitch, yaw and roll, in radians per second squared.
pub const TURN_ACCEL: DVec3 = DVec3::new(2.4, 1.6, 3.6);

/// Turn rates the assist holds at full stick around pitch, yaw and roll, in radians per second.
pub const TURN_RATE: DVec3 = DVec3::new(1.1, 0.7, 1.8);

/// How much harder than the pilot the assist may fire the attitude thrusters.
const ASSIST_GAIN: f64 = 2.5;

/// Acceleration of the vertical thrusters in metres per second squared, with the assist off.
pub const LIFT_ACCEL: f64 = 55.0;

/// Throttle under which the lever counts as neutral.
pub const NEUTRAL: f64 = 0.05;

/// Stick deflection per radian between the roof and the local up while the assist levels the ship.
const LEVEL_GAIN: f64 = 2.0;

/// Returns the spin after `h` seconds of attitude thrusters, for a `stick` of pitch, yaw and roll from -1 to 1.
///
/// Without assist the stick is a torque. With assist it is a turn rate, so a released stick stops the rotation.
pub fn steer(spin: DVec3, stick: DVec3, assist: bool, h: f64) -> DVec3 {
    if !assist {
        return spin + stick * TURN_ACCEL * h;
    }
    let reach = TURN_ACCEL * (ASSIST_GAIN * h);
    spin + (stick * TURN_RATE - spin).clamp(-reach, reach)
}

/// Returns how much of the air is missing over a planet whose air is `thickness` times the
/// home air: all of it without air, none of it in home air or thicker.
///
/// In home air the assist keeps a part of its push across the nose and the ship carries its
/// turns a little wider. The share that is missing gives the rest back, so the ship handles
/// over an airless planet as it does in space.
pub fn airless(thickness: f64) -> f64 {
    (1.0 - thickness).clamp(0.0, 1.0)
}

/// Returns the stick of pitch and roll that brings the roof of a ship with orientation `rot`
/// onto the unit direction `up`. The heading is left alone. With the flight assist the stick
/// is a turn rate, so the ship settles level without swinging past.
pub fn level(rot: DQuat, up: DVec3) -> DVec3 {
    let seen = rot.inverse() * up;
    let axis = DVec3::Y.cross(seen);
    let angle = axis.length().atan2(seen.y);
    let turn = axis.try_normalize().unwrap_or(DVec3::Z);
    turn * (angle * LEVEL_GAIN).min(1.0)
}

/// Seconds a held turn key takes to bring its axis from nothing to full deflection.
pub const RAMP: f64 = 0.35;

/// Returns how far each turn key has been held after `dt` more seconds, from -1 to 1, for keys
/// that ask for `keys` (-1, 0 or 1 per axis). A released key, or one pressed the other way,
/// starts again from nothing.
pub fn ramp(held: DVec3, keys: DVec3, dt: f64) -> DVec3 {
    let one = |held: f64, key: f64| {
        let kept = if held * key > 0.0 { held } else { 0.0 };
        (kept + key * dt / RAMP).clamp(-1.0, 1.0)
    };
    DVec3::new(
        one(held.x, keys.x),
        one(held.y, keys.y),
        one(held.z, keys.z),
    )
}

/// Returns the stick for keys held as far as `held`: it eases in, so that a tap of a tenth of
/// a second deflects the stick by under a tenth and a key held for [`RAMP`] fully.
pub fn eased(held: DVec3) -> DVec3 {
    held * held.abs()
}
