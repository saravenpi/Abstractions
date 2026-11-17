//! The shape of the flight effects: when the air is thick and the fall is fast enough for the hull
//! to glow, and which way the effects stream.

use bevy::prelude::*;

use super::celestial::smoothstep;

/// Dynamic pressure in pascals at which the glow starts and at which it is full.
///
/// The ship flies in an exponential atmosphere whose drag grows with the same pressure, so a
/// ship under power settles at a pressure set by its thrust alone: about 25 kPa at full
/// throttle and 76 kPa under boost, at any height. A ship falling in from orbit with the
/// flight assist on only peaks at 3 to 14 kPa, because it meets the air belly first. The
/// pressure therefore cannot tell the two apart: the sink rate does, see [`GLOW_SINK`]. These
/// bounds are low enough for an entry at half the orbital speed to glow.
const GLOW_PRESSURE: (f32, f32) = (600.0, 5.0e3);

/// Speed toward the ground in metres per second at which the glow starts and at which it is
/// full. Level flight, a descent of one in ten under full boost and a landing approach stay
/// under the first. A fall from the top of the air passes the second.
const GLOW_SINK: (f32, f32) = (30.0, 110.0);

/// Returns the strength of the re-entry glow, from 0 to 1, for a dynamic pressure in pascals
/// and a speed toward the ground in metres per second: thick air met while falling fast.
pub fn entry_glow(pressure: f32, sink: f32) -> f32 {
    smoothstep(GLOW_PRESSURE.0, GLOW_PRESSURE.1, pressure)
        * smoothstep(GLOW_SINK.0, GLOW_SINK.1, sink)
}

/// Returns where the effects stream from: along `velocity`, or along `ahead`, the nose of the
/// ship, without motion and from the first moment the drive spools. In a star frame the
/// velocity still carries the orbit of the planet the ship left, which swept the warp streaks
/// sideways while the drive spooled.
pub fn stream_heading(velocity: Vec3, ahead: Vec3, warp: f32) -> Vec3 {
    if warp > 0.0 {
        return ahead;
    }
    velocity.try_normalize().unwrap_or(ahead)
}
