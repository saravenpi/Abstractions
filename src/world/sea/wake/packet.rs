//! One wake packet: the waves a source left at one place and one moment, and how they spread.
//!
//! A packet holds the same numbers the water shader reads, in three `vec4` rows.
//!
//! Its divergent waves are a ring that grows at the group speed of the waves. The arc of the ring
//! that meets the rings of the entries before it builds an arm of the wedge. Its transverse waves
//! are a stretch of crests between the arms that moves with the source.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::physics::{arm_angle, froude, transverse_share};
use super::tune::{HOVER_WAVELENGTH, MIN_WAVELENGTH, PACKET};

/// The numbers of one wake packet.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Packet {
    /// Clock in seconds when the source laid it.
    pub born: f32,
    /// Speed in metres per second at which the ring grows.
    pub front: f32,
    /// Wave number of the ring waves in radians per metre.
    pub k: f32,
    /// Height in metres of the ring waves.
    pub amp: f32,
    /// Wave number of the transverse waves in radians per metre.
    pub k_t: f32,
    /// Height in metres of the transverse waves.
    pub amp_t: f32,
    /// Speed of the source in metres per second, zero for a source that hovers.
    pub speed: f32,
    /// Size of the source in metres.
    pub size: f32,
    /// Speed in metres per second at which the crests of the ring waves run.
    pub crest: f32,
    /// Half width in metres of the ring.
    pub width: f32,
}

impl Packet {
    /// Returns the packet of a source `size` metres across that moves at `speed` metres per
    /// second under `gravity` and raises waves `amp` metres high at clock `born`.
    ///
    /// The ring grows at the group speed that puts the arms at the angle of the wedge, and its
    /// crests run at the phase speed of deep water waves of its wavelength.
    pub fn moving(speed: f32, gravity: f32, size: f32, (amp, born): (f32, f32)) -> Self {
        let front = arm_angle(speed, gravity, size).sin() * speed;
        let wavelength = (8.0 * PI * front * front / gravity.max(1e-6)).max(MIN_WAVELENGTH);
        let k = TAU / wavelength;
        Self {
            born,
            front,
            k,
            amp,
            k_t: gravity / (speed * speed).max(1e-6),
            amp_t: amp * transverse_share(froude(speed, gravity, size)),
            speed,
            size,
            crest: (gravity / k).sqrt(),
            width: PACKET * wavelength,
        }
    }

    /// Returns the packet of a source `size` metres across that hovers: a ring of waves
    /// [`HOVER_WAVELENGTH`] long that runs out at their phase speed.
    pub fn hover(gravity: f32, size: f32, (amp, born): (f32, f32)) -> Self {
        let k = TAU / HOVER_WAVELENGTH;
        let crest = (gravity.max(0.0) / k).sqrt();
        Self {
            born,
            front: crest,
            k,
            amp,
            size,
            crest,
            width: PACKET * HOVER_WAVELENGTH,
            ..Self::default()
        }
    }

    /// Returns the three rows the shader reads.
    pub fn rows(&self) -> [Vec4; 3] {
        [
            Vec4::new(self.born, self.front, self.k, self.amp),
            Vec4::new(self.k_t, self.amp_t, self.speed, self.size),
            Vec4::new(self.crest, self.width, 0.0, 0.0),
        ]
    }

    /// Returns the packet that the rows `[p, q, e]` of [`Packet::rows`] hold.
    pub fn from_rows([p, q, e]: [Vec4; 3]) -> Self {
        Self {
            born: p.x,
            front: p.y,
            k: p.z,
            amp: p.w,
            k_t: q.x,
            amp_t: q.y,
            speed: q.z,
            size: q.w,
            crest: e.x,
            width: e.y,
        }
    }
}
