//! The wake entries packed for a shader: one block for the sea, in sea coordinates, and one for
//! the foam, in its own space.
//!
//! A shader loops over the first `count` entries only, so a sea with no live wake costs one test.

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

use super::packet::Packet;
use super::tune::{FOOT, WAKES};
use super::wakes::Wakes;

/// Rows of one entry in the block of the sea: the place and direction, then the packet.
pub const SEA_ROWS: usize = 4;

/// Rows of one entry in the block of the foam: the place, direction and up, then the packet.
pub const FOAM_ROWS: usize = 6;

/// The wake entries as the water shader reads them, `SeaWakes` in `field.wesl`.
///
/// Each entry is a row of sea coordinates `(x, z, dir x, dir z)` and the three rows of its packet.
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct SeaWakes {
    /// Number of live entries in x.
    pub count: Vec4,
    /// The rows of the live entries, packed from the first.
    pub rows: [Vec4; SEA_ROWS * WAKES],
}

/// The wake entries as the foam shader reads them, `FoamWakes` in `field.wesl`: a row of render position, a row of direction,
/// a row of up and the three rows of the packet.
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct FoamWakes {
    /// Number of live entries in x.
    pub count: Vec4,
    /// The rows of the live entries, packed from the first.
    pub rows: [Vec4; FOAM_ROWS * WAKES],
}

impl Default for SeaWakes {
    /// Returns a block with no entry.
    fn default() -> Self {
        Self {
            count: Vec4::ZERO,
            rows: [Vec4::ZERO; SEA_ROWS * WAKES],
        }
    }
}

impl Default for FoamWakes {
    /// Returns a block with no entry.
    fn default() -> Self {
        Self {
            count: Vec4::ZERO,
            rows: [Vec4::ZERO; FOAM_ROWS * WAKES],
        }
    }
}

impl SeaWakes {
    /// Packs the entries of `wakes` that show at clock `now`, placing each with `locate`, which
    /// maps a frame position to sea coordinates. A buffer with none live gives the empty block.
    pub fn pack(wakes: &Wakes, now: f32, locate: impl Fn(DVec3) -> Vec2) -> Self {
        let mut block = Self::default();
        for (slot, wake) in wakes.live(now).enumerate() {
            let at = locate(wake.at);
            let dir = (locate(wake.ahead) - at).normalize_or(Vec2::X);
            let base = slot * SEA_ROWS;
            block.rows[base] = Vec4::new(at.x, at.y, dir.x, dir.y);
            block.rows[base + 1..base + SEA_ROWS].copy_from_slice(&wake.packet.rows());
            block.count.x = (slot + 1) as f32;
        }
        block
    }

    /// Returns the height in metres of the wake waves at sea coordinates `p` at clock `now`, as
    /// the water shader raises the sea there.
    pub fn height(&self, p: Vec2, now: f32) -> f32 {
        let live = self.count.x as usize;
        self.rows[..live * SEA_ROWS]
            .as_chunks::<SEA_ROWS>()
            .0
            .iter()
            .map(|row| {
                let rel = p - Vec2::new(row[0].x, row[0].y);
                let dir = Vec2::new(row[0].z, row[0].w);
                let side = dir.perp();
                let packet = Packet::from_rows([row[1], row[2], row[3]]);
                packet.bit((rel.dot(dir), rel.dot(side)), now, FOOT).x
            })
            .sum()
    }
}

impl FoamWakes {
    /// Packs the entries of `wakes` that show at clock `now`, placing each with `locate`, which
    /// maps a frame position to the local space of the foam.
    pub fn pack(wakes: &Wakes, now: f32, locate: impl Fn(DVec3) -> Vec3) -> Self {
        let mut block = Self::default();
        for (slot, wake) in wakes.live(now).enumerate() {
            let at = locate(wake.at);
            let dir = (locate(wake.ahead) - at).normalize_or(Vec3::X);
            let up = (locate(wake.over) - at).normalize_or(Vec3::Y);
            let base = slot * FOAM_ROWS;
            block.rows[base] = at.extend(0.0);
            block.rows[base + 1] = dir.extend(0.0);
            block.rows[base + 2] = up.extend(0.0);
            block.rows[base + 3..base + FOAM_ROWS].copy_from_slice(&wake.packet.rows());
            block.count.x = (slot + 1) as f32;
        }
        block
    }
}
