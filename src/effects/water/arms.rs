//! The foam of one segment of a wake: a turbulent band along the track and the two arms of the V.
//!
//! The arms are the envelope of rings that grow from every point of the track. A ring grows at
//! the group speed of its waves, the sine of the half angle of the wedge times the speed of the
//! source, and the arm stands where the ring touches the wedge: a step of that speed from the
//! track, leaning forward of the side by the half angle.

use bevy::prelude::*;

use super::foam::Quad;
use super::throw::{Mark, Spot};
use super::tune::{ARM_BAND, ARM_SHARE, ARM_WIDEN, FOAM_LIFE};
use super::wash::spread;
use crate::render::smoothstep;
use crate::world::wake::arm_angle;

/// What one segment of a wake needs to know about the pass of its source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pass {
    /// Speed of the source over the sea in metres per second.
    pub speed: f32,
    /// Size of the source in metres.
    pub size: f32,
    /// Strength of its spray, from 0 to 1.
    pub strength: f32,
    /// Half angle of the arms of its wake in radians.
    pub angle: f32,
    /// How close it flies to the water, from 0 far over it to 1 touching it.
    pub touch: f32,
}

impl Pass {
    /// Returns the pass of `spot` that sprays with `strength`.
    pub fn of(spot: &Spot, strength: f32) -> Self {
        let speed = spot.velocity.length();
        Self {
            speed,
            size: spot.size,
            strength,
            angle: arm_angle(speed, spot.gravity, spot.size),
            touch: 1.0 - smoothstep(0.5 * spot.size, 2.0 * spot.size, spot.height.max(0.0)),
        }
    }
}

/// Returns the three quads of the segment between `ends` and the two marks that bound it: the
/// centre band, then the two arms.
pub fn quads(ends: [Vec3; 2], marks: [&Mark; 2], pass: &Pass) -> [Quad; 3] {
    let (sin, cos) = pass.angle.sin_cos();
    let out = sin * pass.speed;
    let births = marks.map(|mark| mark.birth);
    let arm = |sign: f32| Quad {
        ends,
        dirs: marks.map(|mark| mark.side.cross(mark.up) * sin + mark.side * (sign * cos)),
        shape: Vec4::new(0.0, out, ARM_BAND, ARM_WIDEN * out),
        births,
        fade: Vec2::new(pass.strength * ARM_SHARE, FOAM_LIFE),
    };
    let centre = Quad {
        ends,
        dirs: marks.map(|mark| mark.side),
        shape: Vec4::new(0.0, 0.0, 0.5 * pass.size, spread(pass.speed)),
        births,
        fade: Vec2::new(pass.strength * pass.touch, FOAM_LIFE),
    };
    [centre, arm(1.0), arm(-1.0)]
}
