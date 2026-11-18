//! The dust of a warp: a sparse field of near grains of the interstellar medium that the ship
//! flies through. A real flight at a steady speed shows a still, aberrated sky, which reads as
//! frozen; near grains rushing past give the eye the motion it expects. `starfield.wesl` places
//! every grain by these rules on the GPU, from a hash of the grain and of its lap along the
//! line of flight, so nothing is allocated per frame; here they are written again to be tested.

#[cfg(test)]
use bevy::prelude::*;

use super::Sequence;
use crate::render::celestial::smoothstep;

/// Grains in the field: one quad each in a mesh built once.
pub const MOTES: u32 = 2048;

/// Distances along the heading, in radii of the tube of grains, at which a grain starts its lap
/// ahead and ends it behind the camera.
#[cfg(test)]
pub const DEPTH: (f32, f32) = (8.0, -2.0);

/// Least and most distance of a grain from the line of flight, in radii of the tube: none
/// passes through the camera.
#[cfg(test)]
pub const TUBE: (f32, f32) = (0.12, 1.0);

/// Laps per second the grains run at the top warp speed.
pub const LAPS: f32 = 0.8;

/// Share of that rate the flow keeps while the drive only spools: the bubble forms and the
/// grains start to creep before the ship gathers speed.
pub const CREEP: f32 = 0.15;

/// Laps after which the phase of the flow wraps. It is the period of the lap hash, so the
/// wrap changes no grain.
pub const WRAP: f32 = 1024.0;

/// Shares of its lap over which a grain fades in far ahead and out behind the camera.
#[cfg(test)]
pub const FADE: (f32, f32) = (0.2, 0.85);

impl Sequence {
    /// Returns the laps per second the dust of the warp runs: nothing outside a warp, a creep
    /// while the drive spools, then rising with the velocity up to [`LAPS`] at the top speed.
    pub fn flow(&self) -> f32 {
        LAPS * smoothstep(0.0, 1.0, self.charge) * (CREEP + (1.0 - CREEP) * self.velocity)
    }

    /// Returns the share of the dust of the warp that is in, from 0 outside a warp to 1 once
    /// the drive has engaged: it gathers through the spool and thins through the drop-out.
    pub fn dust(&self) -> f32 {
        smoothstep(0.0, 1.0, self.charge.max(self.velocity))
    }
}

/// Returns a value from 0 to 1 that only depends on `key`: the hash `starfield.wesl` uses too.
#[cfg(test)]
pub fn hash(key: u32) -> f32 {
    let mut h = key.wrapping_mul(0x27d4_eb2d);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h = (h ^ (h >> 12)).wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h >> 8) as f32 / 16_777_216.0
}

/// One grain as the camera sees it this frame.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mote {
    /// Offset from the camera, in radii of the tube, in render axes.
    pub offset: Vec3,
    /// Share of the full light of a grain it shows, from 0 to 1.
    pub light: f32,
    /// How far through its lap it is, from 0 far ahead to 1 behind the camera.
    pub lap: f32,
}

/// Returns how much of its light a grain shows at the share `t` of its lap: nothing at either
/// end, so the jump from behind the camera back to far ahead is never seen.
#[cfg(test)]
pub fn window(t: f32) -> f32 {
    let rise = (t / FADE.0).clamp(0.0, 1.0);
    let fall = ((1.0 - t) / (1.0 - FADE.1)).clamp(0.0, 1.0);
    rise * rise * (3.0 - 2.0 * rise) * fall * fall * (3.0 - 2.0 * fall)
}

/// Returns two unit vectors at right angles to the unit vector `axis` and to each other, picked
/// the way `starfield.wesl` picks them.
#[cfg(test)]
pub fn sides(axis: Vec3) -> (Vec3, Vec3) {
    let pick = if axis.y.abs() >= 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let side = axis.cross(pick).normalize();
    (side, axis.cross(side))
}

/// Returns grain `index` of the field when the flow has run `phase` laps along the unit vector
/// `heading`, with the share `density` of the field in.
///
/// Each grain starts at its own point of the lap. At each lap it comes back far ahead at a new
/// place across the tube, hashed from the grain and the lap, while it is dark. The field thins
/// grain by grain as the density falls, each fading rather than popping.
#[cfg(test)]
pub fn mote(index: u32, phase: f32, density: f32, heading: Vec3) -> Mote {
    let run = hash(index.wrapping_mul(4)) + phase;
    let lap = (run.floor() as i32 & 1023) as u32;
    let key = (index + MOTES * (lap + 1)).wrapping_mul(4);
    let t = run - run.floor();
    let reach = TUBE.0 * TUBE.0 + (TUBE.1 * TUBE.1 - TUBE.0 * TUBE.0) * hash(key + 1);
    let turn = std::f32::consts::TAU * hash(key + 2);
    let (side, rise) = sides(heading);
    let across = (side * turn.cos() + rise * turn.sin()) * reach.sqrt();
    let offset = heading * (DEPTH.0 + (DEPTH.1 - DEPTH.0) * t) + across;
    let thin = ((density * 1.5 - hash(key + 3)) * 2.0).clamp(0.0, 1.0);
    let light = window(t) * thin / (1.0 + offset.length_squared());
    Mote {
        offset,
        light,
        lap: t,
    }
}

/// Returns how far the stars sway out along their streaks at the share `c` of their cycle,
/// from 0 to 1: a slow slide out, then an eased return, with no step anywhere.
#[cfg(test)]
pub fn crawl(c: f32) -> f32 {
    let c = c - c.floor();
    if c < 0.8 {
        return c / 0.8;
    }
    let back = ((c - 0.8) / 0.2).clamp(0.0, 1.0);
    1.0 - back * back * (3.0 - 2.0 * back)
}
