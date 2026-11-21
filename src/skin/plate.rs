//! Plates in tables: the chamfered boxes a body wears, written as rows of numbers.

use bevy::prelude::*;

use super::Rgb;
use super::block::{Block, block};
use super::detail::{Detail, SMALL};
use super::soup::Soup;

/// One chamfered box of a table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plate {
    /// Middle of the box.
    pub at: Vec3,
    /// Half extents before the turn.
    pub half: Vec3,
    /// Turn about the X, Y and Z axes in radians, applied as a roll about Z, then a pitch about
    /// X, then a yaw about Y.
    pub turn: Vec3,
    /// Chamfer in meters.
    pub cut: f32,
    /// Colour of the faces, then of the chamfers.
    pub tones: [Rgb; 2],
    /// Whether it is too small to build from afar.
    pub fine: bool,
}

impl Plate {
    /// Returns a plate at `at` of half extents `half`, with the small chamfer, faces and
    /// chamfers in `tones`.
    pub const fn new(at: [f32; 3], half: [f32; 3], tones: [Rgb; 2]) -> Self {
        Self {
            at: Vec3::from_array(at),
            half: Vec3::from_array(half),
            turn: Vec3::ZERO,
            cut: SMALL,
            tones,
            fine: false,
        }
    }

    /// Returns the plate rolled by `z` radians, then pitched by `x`, then yawed by `y`.
    pub const fn turned(mut self, x: f32, y: f32, z: f32) -> Self {
        self.turn = Vec3::new(x, y, z);
        self
    }

    /// Returns the plate with the chamfer `cut`, in meters. Zero leaves a plain box.
    pub const fn cut(mut self, cut: f32) -> Self {
        self.cut = cut;
        self
    }

    /// Returns the plate marked as too small to build from afar.
    pub const fn fine(mut self) -> Self {
        self.fine = true;
        self
    }
}

/// Adds the plates of `list` at the level `detail`.
pub fn plates(soup: &mut Soup, list: &[Plate], detail: Detail) {
    for plate in list.iter().filter(|plate| detail.near() || !plate.fine) {
        let turn = Quat::from_euler(EulerRot::YXZ, plate.turn.y, plate.turn.x, plate.turn.z);
        let shape = Block::new(plate.at, plate.half, detail.cut(plate.cut)).turned(turn);
        block(soup, &shape, plate.tones[0], plate.tones[1]);
    }
}
