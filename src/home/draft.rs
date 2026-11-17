//! The draft of the base: what the pieces draw into, each in its own level frame on the lot.

use bevy::math::Affine3A;
use bevy::prelude::*;

use super::glow::Glow;
use super::lot::Lot;
use super::soup::Soup;
use super::tune::Spot;

/// A solid box nothing walks through, in the frame of the base.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    /// Centre of the box.
    pub at: Vec3,
    /// Turn of the box around the vertical.
    pub rot: Quat,
    /// Width, height and depth of the box.
    pub size: Vec3,
}

/// The base while it is drawn. Pieces draw in the frame set by [`Draft::stand`]; everything is
/// kept in the frame of the base.
pub struct Draft<'a> {
    /// The ground the base stands on.
    pub(super) lot: &'a Lot,
    /// Pose of the piece being drawn, in the frame of the base.
    pub(super) pose: Affine3A,
    /// Everything lit by the sun and the lamps.
    pub lit: Soup,
    /// The glowing parts, one soup per [`Glow`].
    pub glows: [Soup; 6],
    /// Boxes nothing walks through.
    pub blocks: Vec<Block>,
    /// Where each leg and each thing lying on the ground meets the ground.
    pub feet: Vec<Vec3>,
    /// Three corners of the top of every level platform.
    pub decks: Vec<[Vec3; 3]>,
    /// Where each light that is cast sits.
    pub lamps: Vec<(Glow, Vec3)>,
}

impl<'a> Draft<'a> {
    /// Starts an empty draft on `lot`, drawing in the frame of the base.
    pub fn new(lot: &'a Lot) -> Self {
        Self {
            lot,
            pose: Affine3A::IDENTITY,
            lit: Soup::default(),
            glows: Default::default(),
            blocks: Vec::new(),
            feet: Vec::new(),
            decks: Vec::new(),
            lamps: Vec::new(),
        }
    }

    /// Moves the drawing frame to a piece standing at `spot`: level, its origin `clear` metres
    /// above the highest ground under its `feet`. Without feet the origin is level with the
    /// base centre.
    pub fn stand(&mut self, spot: Spot, feet: &[Vec2], clear: f32) {
        self.pose = self.lot.stand(spot, feet, clear);
    }

    /// Returns where the point `p` of the piece lies in the frame of the base.
    pub fn at(&self, p: Vec3) -> Vec3 {
        self.pose.transform_point3(p)
    }

    /// Returns the height of the ground in the frame of the piece, under its point `at`, given
    /// right and back of its origin.
    pub fn ground(&self, at: Vec2) -> f32 {
        let root = self.at(Vec3::new(at.x, 0.0, at.y));
        self.lot.ground(root.x, root.z) - self.pose.translation.y
    }

    /// Adds what `paint` draws in the frame of the piece to the lit part of the base.
    pub fn draw(&mut self, paint: impl FnOnce(&mut Soup)) {
        let mut soup = Soup::default();
        paint(&mut soup);
        self.lit.merge(&soup, self.pose);
    }

    /// Adds what `paint` draws in the frame of the piece to the glowing part `glow`.
    pub fn shine(&mut self, glow: Glow, paint: impl FnOnce(&mut Soup)) {
        let mut soup = Soup::default();
        paint(&mut soup);
        self.glows[glow as usize].merge(&soup, self.pose);
    }
}
