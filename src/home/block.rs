//! The solid boxes of the base. Every box and every bar that is drawn is one, so nothing drawn can lack
//! its collider: only what is too thin to run into, what lies in the ground and what sits inside a larger
//! box is left out.

use bevy::math::Affine3A;
use bevy::prelude::*;

/// Longest side in metres from which a drawn box is solid, and the width its middle side must reach:
/// thinner bars are cables, rods and the legs of furniture.
pub const BULK: (f32, f32) = (0.3, 0.1);

/// Height in metres above the lowest ground under it up to which a drawn box is carried by the ground
/// itself, like the slabs of the path, and distance by which a box may stick out of a larger one and still
/// be left to it.
pub const FLUSH: (f32, f32) = (0.3, 0.1);

/// A solid box nothing passes through.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    /// Centre of the box.
    pub at: Vec3,
    /// Turn of the box.
    pub rot: Quat,
    /// Width, height and depth of the box.
    pub size: Vec3,
}

impl Block {
    /// Returns the upright box between two corners.
    pub fn between(lo: Vec3, hi: Vec3) -> Self {
        Self {
            at: (lo + hi) * 0.5,
            rot: Quat::IDENTITY,
            size: (hi - lo).abs(),
        }
    }

    /// Returns the box of a bar from `a` to `b` whose section has the sides `side` and `other`, both at
    /// right angles to the bar and half as long as the bar is wide, with `other` along the bar crossed with `side`.
    pub fn along(a: Vec3, b: Vec3, side: Vec3, other: Vec3) -> Self {
        let axes = Mat3::from_cols(other.normalize(), (b - a).normalize(), side.normalize());
        Self {
            at: (a + b) * 0.5,
            rot: Quat::from_mat3(&axes).normalize(),
            size: Vec3::new(other.length() * 2.0, (b - a).length(), side.length() * 2.0),
        }
    }

    /// Returns the box moved by `pose`.
    pub fn moved(self, pose: Affine3A) -> Self {
        Self {
            at: pose.transform_point3(self.at),
            rot: (Quat::from_mat3a(&pose.matrix3) * self.rot).normalize(),
            size: self.size,
        }
    }

    /// Returns true when the box is large enough to run into.
    pub fn bulky(&self) -> bool {
        let mut sides = self.size.to_array();
        sides.sort_by(f32::total_cmp);
        sides[2] >= BULK.0 && sides[1] >= BULK.1
    }

    /// Returns the eight corners of the box.
    pub fn corners(&self) -> [Vec3; 8] {
        std::array::from_fn(|i| {
            let sign = Vec3::new((i & 1) as f32, (i >> 1 & 1) as f32, (i >> 2 & 1) as f32) - 0.5;
            self.at + self.rot * (self.size * sign)
        })
    }

    /// Returns true when no corner of `other` sticks out of this box by more than `slack` metres.
    pub fn holds(&self, other: &Block, slack: f32) -> bool {
        let back = self.rot.inverse();
        let inside = |corner: Vec3| {
            let local = (back * (corner - self.at)).abs();
            local.cmple(self.size * 0.5 + slack).all()
        };
        other.corners().into_iter().all(inside)
    }
}

/// Returns `blocks` without those a larger one holds: the plates, frames and panels drawn on a box are
/// left to that box.
pub fn pruned(blocks: Vec<Block>) -> Vec<Block> {
    let volume = |block: &Block| block.size.element_product();
    let held = |(i, block): (usize, &Block)| {
        let mut others = blocks.iter().enumerate().filter(|(j, _)| *j != i);
        let larger = |(j, other): (usize, &Block)| {
            let first = volume(other) > volume(block) || (volume(other) == volume(block) && j < i);
            first && other.holds(block, FLUSH.1)
        };
        others.any(larger)
    };
    let keep: Vec<bool> = blocks.iter().enumerate().map(|b| !held(b)).collect();
    let kept = blocks.iter().zip(keep).filter(|(_, keep)| *keep);
    kept.map(|(block, _)| *block).collect()
}
