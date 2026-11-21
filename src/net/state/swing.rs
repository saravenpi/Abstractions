//! The hands of a robot on the wire: what they do with the tool and for how long, packed in two
//! fields of the robot record and unpacked into the strike state the hands pose from. The three
//! high bits of the act carry how far up or down the strike aims, inside protocol 3: a record that
//! leaves them clear aims straight ahead, as before.

use bevy::prelude::*;

use crate::avatar::strike::{Act, Dir, Strike};

/// Lowest and highest elevation of an aim on the wire, from the right shoulder, in radians.
const ELEVATION: (f32, f32) = (-1.4, 0.6);

/// Steps of the elevation of an aim on the wire: 1 to 7, 0 for no aim.
const STEPS: f32 = 6.0;

/// Metres from the right shoulder at which an aim from the wire is put.
const AIM_AT: f32 = 1.2;

/// Where the right shoulder of a body at rest is, in the frame of the body.
const SHOULDER: Vec3 = Vec3::new(0.212, 0.38, 0.0);

/// Returns the three bits of the elevation of `aim` seen from the shoulder, 0 for none.
fn rise(aim: Option<Vec3>) -> u8 {
    let Some(aim) = aim else { return 0 };
    let d = aim - SHOULDER;
    let up = d.y.atan2(Vec2::new(d.x, d.z).length().max(1e-3));
    let share = (up - ELEVATION.0) / (ELEVATION.1 - ELEVATION.0);
    1 + (share.clamp(0.0, 1.0) * STEPS).round() as u8
}

/// Returns the aim of three bits: straight ahead of the shoulder at its elevation, or none.
fn aim_of(bits: u8) -> Option<Vec3> {
    let step = f32::from(bits & 7);
    (step > 0.0).then(|| {
        let up = ELEVATION.0 + (step - 1.0) / STEPS * (ELEVATION.1 - ELEVATION.0);
        SHOULDER + Vec3::new(0.0, up.sin(), -up.cos()) * AIM_AT
    })
}

/// Returns the side `dir` as its two bits.
fn side(dir: Dir) -> u8 {
    match dir {
        Dir::Right => 0,
        Dir::Left => 1,
        Dir::Up => 2,
        Dir::Down => 3,
    }
}

/// Returns the side of two bits.
fn dir_of(bits: u8) -> Dir {
    match bits & 3 {
        0 => Dir::Right,
        1 => Dir::Left,
        2 => Dir::Up,
        _ => Dir::Down,
    }
}

/// Returns the act of `strike` as its rank with the side in bits 3 and 4 and the elevation of its
/// aim in bits 5 to 7, and the seconds it has lasted.
pub fn pack(strike: &Strike) -> (u8, f32) {
    let (rank, phase) = match strike.act {
        Act::Ready => (0, 0.0),
        Act::Windup(t) => (1, t),
        Act::Swing(t) => (2, t),
        Act::Recover(t) => (3, t),
        Act::Guard(t) => (4, t),
        Act::Draw(t) => (5, t),
        Act::Stun(left, _) => (6, left),
        Act::Stow(t) => (7, t),
    };
    (rank | side(strike.dir) << 3 | rise(strike.aim) << 5, phase)
}

/// Returns the strike state a record of `swing` and `phase` stands for: the act, the side and the
/// aim, with nothing landed and no blade yet.
pub fn unpack(swing: u8, phase: f32) -> Strike {
    let act = match swing & 7 {
        1 => Act::Windup(phase),
        2 => Act::Swing(phase),
        3 => Act::Recover(phase),
        4 => Act::Guard(phase),
        5 => Act::Draw(phase),
        6 => Act::Stun(phase, 0.0),
        7 => Act::Stow(phase),
        _ => Act::Ready,
    };
    Strike {
        act,
        dir: dir_of(swing >> 3),
        aim: aim_of(swing >> 5),
        ..Strike::default()
    }
}
