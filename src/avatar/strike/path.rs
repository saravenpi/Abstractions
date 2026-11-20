//! Paths: the way the grip and the tool travel through a swing or a thrust, from the cocked key to
//! the follow-through.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::dir::Dir;
use super::held::Style;
use super::key::Key;
use super::tune::{GRIP_RADIUS, SPINE, TWIST};

/// The way a strike travels, in the frame of the torso.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Path {
    /// A swing: the tool pivots at the grip while the grip sweeps a circle, the tool along its radius.
    Sweep {
        /// Centre of the circle the grip sweeps.
        centre: Vec3,
        /// Unit radius at angle 0.
        u: Vec3,
        /// Unit radius at a quarter turn.
        v: Vec3,
        /// Angles at the cocked key and at the follow-through, in radians.
        turn: [f32; 2],
        /// Chest turn and bend at the follow-through; the cocked key holds their opposites.
        trunk: [f32; 2],
    },
    /// A thrust: the grip drives straight ahead along the tool.
    Line {
        /// Grip at the cocked key and at full reach.
        grip: [Vec3; 2],
        /// Unit direction of the tool.
        axis: Vec3,
        /// Chest turn and bend at full reach.
        trunk: [f32; 2],
    },
}

/// Centre of the grip circle, radius at angle 0 and trunk at the end of a swing, per [`Dir`].
const SWEEPS: [(Vec3, Vec3, [f32; 2]); 4] = [
    (
        Vec3::new(0.18, 0.3, -0.1),
        Vec3::new(1.0, 0.25, 0.0),
        [TWIST, 0.1],
    ),
    (
        Vec3::new(0.12, 0.3, -0.1),
        Vec3::new(-1.0, 0.25, 0.0),
        [-TWIST, 0.1],
    ),
    (
        Vec3::new(0.15, 0.35, -0.1),
        Vec3::new(0.15, 1.0, 0.0),
        [0.15, SPINE],
    ),
    (
        Vec3::new(0.15, 0.25, -0.1),
        Vec3::new(0.3, -1.0, 0.0),
        [0.15, -0.15],
    ),
];

/// Returns the swing from `dir`: its plane, the centre of the grip circle and the trunk at its end.
fn sweep(dir: Dir) -> Path {
    let (centre, u, trunk) = SWEEPS[dir as usize];
    let u = u.normalize();
    let v = Vec3::NEG_Z.reject_from(u).normalize();
    Path::Sweep {
        centre,
        u,
        v,
        turn: [-0.5, FRAC_PI_2 + 0.9],
        trunk,
    }
}

/// Returns the thrust: high when the mouse asks from above, else at chest height.
fn line(dir: Dir) -> Path {
    let rise = if dir == Dir::Up { 0.3 } else { 0.05 };
    Path::Line {
        grip: [Vec3::new(0.2, 0.2, -0.02), Vec3::new(0.2, 0.28, -0.4)],
        axis: Vec3::new(0.0, rise, -1.0).normalize(),
        trunk: [0.3, 0.15],
    }
}

impl Path {
    /// Returns the path of a strike of `style` from `dir`. A chop comes overhead unless it is asked
    /// from a side.
    pub fn of(style: Style, dir: Dir) -> Self {
        match (style, dir) {
            (Style::Thrust, _) => line(dir),
            (Style::Chop, Dir::Down) => sweep(Dir::Up),
            _ => sweep(dir),
        }
    }

    /// Returns how far the path travels: radians of a swing, metres of a thrust.
    pub fn length(&self) -> f32 {
        match *self {
            Path::Sweep { turn, .. } => turn[1] - turn[0],
            Path::Line { grip, .. } => grip[0].distance(grip[1]),
        }
    }

    /// Returns the key at the share `s` of the path, 0 cocked to 1 followed through.
    pub fn at(&self, s: f32) -> Key {
        let s = s.clamp(0.0, 1.0);
        let swing = 2.0 * s - 1.0;
        match *self {
            Path::Sweep {
                centre,
                u,
                v,
                turn,
                trunk,
            } => {
                let angle = turn[0] + (turn[1] - turn[0]) * s;
                let radius = angle.cos() * u + angle.sin() * v;
                Key {
                    grip: centre + GRIP_RADIUS * radius,
                    axis: radius,
                    twist: trunk[0] * swing,
                    spine: trunk[1] * swing,
                }
            }
            Path::Line { grip, axis, trunk } => Key {
                grip: grip[0].lerp(grip[1], s),
                axis,
                twist: trunk[0] * swing,
                spine: trunk[1] * swing,
            },
        }
    }
}
