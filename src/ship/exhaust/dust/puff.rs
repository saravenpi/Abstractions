//! One puff of dust: thrown out along the ground by a jet, carried by the air that slows to the
//! wind, settling toward the centre of the planet, and held on the ground. It lives in the frame
//! of the planet, so a rebase of the render anchor never moves it over the ground.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::{DRAG_TIME, FLING, LIFE, RISE, SIZE};
use crate::render::smoothstep;
use crate::space::Anchor;

/// One pooled puff, in the frame of the planet.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct Mote {
    /// Its centre in metres from the centre of the planet.
    pub at: DVec3,
    /// Velocity of the air that carries it, in frame axes.
    pub flow: Vec3,
    /// Speed at which it settles through that air.
    pub settle: f32,
    /// Distance from the centre of the planet of the ground under it.
    pub floor: f64,
    /// Where it was when the ground under it was last read.
    pub read: DVec3,
    /// Seconds since it was thrown.
    pub age: f32,
    /// Seconds it lasts. Zero while parked.
    pub life: f32,
    /// Its size in metres when thrown.
    pub size: f32,
}

/// Returns a number in `[0, 1)` for the `serial`th puff and a `salt`: the same on every run.
pub fn noise(serial: u32, salt: u32) -> f32 {
    let mut x = serial.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    (x >> 8) as f32 / 16_777_216.0
}

/// Returns the `serial`th puff thrown from the ground at `at` by a jet that still blows there with
/// `strength`, settling at `settle`: out along the ground in its own direction, as fast as the
/// jet still blows, and a little up in its turbulence.
pub fn fling(at: DVec3, (serial, strength): (u32, f32), settle: f32) -> Mote {
    let up = at.normalize_or_zero().as_vec3();
    let (a, b) = up.any_orthonormal_pair();
    let turn = noise(serial, 1) * std::f32::consts::TAU;
    let out = a * turn.cos() + b * turn.sin();
    let fast = FLING * strength * (0.6 + 0.8 * noise(serial, 2));
    let size = SIZE.0 * (0.7 + 0.6 * noise(serial, 3));
    Mote {
        at: at + ((out * 0.5 + up * 0.3) * size).as_dvec3(),
        flow: out * fast + up * (fast * RISE * noise(serial, 4)),
        settle,
        floor: at.length(),
        read: at,
        age: 0.0,
        life: LIFE.0 + (LIFE.1 - LIFE.0) * noise(serial, 5),
        size,
    }
}

/// Reads the ground under `mote` again from `floor`, the distance from the centre of the planet
/// of the ground in each unit direction, once it drifted `far` metres from where it last did.
pub fn reread(mote: &mut Mote, far: f64, floor: impl Fn(DVec3) -> f64) {
    if mote.at.distance(mote.read) > far {
        mote.floor = floor(mote.at.normalize_or_zero());
        mote.read = mote.at;
    }
}

/// Advances a live puff by `dt` seconds in the wind `wind`, in frame axes. The air that carries it
/// slows from the flow of the jet to the wind along the ground over [`DRAG_TIME`], it settles
/// through that air toward the centre of the planet, and the ground stops it: it never sinks
/// under the ground, and the air no longer pushes it down there.
pub fn step(mote: &mut Mote, wind: Vec3, dt: f32) {
    let up = mote.at.normalize_or_zero();
    let level = up.as_vec3();
    let breeze = wind - level * wind.dot(level);
    mote.flow = breeze + (mote.flow - breeze) * (-dt / DRAG_TIME).exp();
    let velocity = mote.flow.as_dvec3() - up * f64::from(mote.settle);
    let next = mote.at + velocity * f64::from(dt);
    let low = next.length() < mote.floor;
    mote.at = match low {
        true => next.normalize_or_zero() * mote.floor,
        false => next,
    };
    if low {
        mote.flow -= level * mote.flow.dot(level).min(0.0);
    }
    mote.age += dt;
}

/// Returns the drawn size of a puff in metres: it swells out of nothing, widens as it spreads
/// and thins to nothing over the end of its life.
pub fn size(mote: &Mote) -> f32 {
    let spent = (mote.age / mote.life.max(1e-3)).clamp(0.0, 1.0);
    let wide = mote.size + SIZE.1 * mote.age.max(0.0).sqrt();
    wide * smoothstep(0.0, 0.08, spent) * (1.0 - smoothstep(0.6, 1.0, spent))
}

/// Returns where a puff is drawn under `anchor`: at its place on the planet, turned by `turn` to
/// face the camera and the same size across every way, so that it is a round puff and never a
/// streak, however fast it moves.
pub fn pose(anchor: &Anchor, turn: Quat, mote: &Mote) -> Transform {
    Transform {
        translation: anchor.to_render(mote.at),
        rotation: turn,
        scale: Vec3::splat(size(mote)),
    }
}
