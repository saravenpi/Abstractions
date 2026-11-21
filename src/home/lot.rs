//! The lot: the flat frame of the base, and the real ground seen from it.

use std::sync::Arc;

use bevy::math::{Affine3A, DMat3, DVec2, DVec3};
use bevy::prelude::*;

use super::tune::Spot;
use crate::world::globe::{Chart, Tangent};
use crate::world::{Globe, Terrain};

/// Passes that walk a point of the flat frame onto the ground along the local vertical.
const PASSES: u32 = 3;

/// Metres around a point within which [`Lot::crest`] reads the ground, so a crease of the 2 m terrain mesh between
/// two points of a flat piece does not rise through it.
const REACH: f32 = 0.9;

/// Directions around a point at which [`Lot::crest`] reads the ground, besides the point itself.
const SPREAD: [[f32; 2]; 8] = [
    [1.0, 0.0],
    [-1.0, 0.0],
    [0.0, 1.0],
    [0.0, -1.0],
    [1.0, 1.0],
    [-1.0, 1.0],
    [1.0, -1.0],
    [-1.0, -1.0],
];

/// Returns the view of `globe` through the chart of the cube face that holds the planet position
/// `pos`, rooted near it: the grid of that chart is the one the chunk meshes the game draws there
/// are built on.
fn drawn(globe: &Arc<Globe>, pos: DVec3) -> Terrain {
    Terrain::near(globe, pos)
}

/// The frame of the base: origin on the ground at the base centre, +x east, +y up there and
/// +z south, with the metric view of the ground around it.
///
/// The frame is flat and the ground is a sphere, so heights go through [`Lot::ground`].
#[derive(Clone)]
pub struct Lot {
    /// Metric view of the ground, centred on the base.
    view: Terrain,
    /// The base centre on the ground, in metres from the planet centre.
    origin: DVec3,
    /// Columns: the east, up and south directions of the frame in planet axes.
    axes: DMat3,
}

impl Lot {
    /// Builds the lot centred on direction `dir` of `globe`, with +x along `east`.
    pub fn new(globe: &Arc<Globe>, dir: DVec3, east: DVec3) -> Self {
        let tangent = Tangent::new(dir, east, globe.radius(), DVec2::ZERO);
        let view = Terrain::on(globe.clone(), Chart::Tangent(tangent));
        let centre = view.embed(0.0, 0.0, 0.0);
        let drawn = drawn(globe, centre);
        let at = drawn.locate(centre);
        Self {
            origin: drawn.embed(at.x, drawn.mesh_height(at.x, at.z), at.z),
            axes: DMat3::from_cols(tangent.east, tangent.origin, -tangent.north),
            view,
        }
    }

    /// Returns the metric view of the ground around the base. Its `(x, z)` are metres east and
    /// south of the base centre.
    pub fn view(&self) -> &Terrain {
        &self.view
    }

    /// Returns the east, up and south directions of the frame in planet axes, as columns.
    pub fn axes(&self) -> DMat3 {
        self.axes
    }

    /// Returns the position in metres from the planet centre of the point `local` of the frame.
    pub fn planet(&self, local: Vec3) -> DVec3 {
        self.origin + self.axes * local.as_dvec3()
    }

    /// Returns the height in the frame of the drawn ground under the frame point `(x, z)`: the
    /// terrain mesh of the chunks, which lives on the cube face charts, not the grid of the view.
    pub fn ground(&self, x: f32, z: f32) -> f32 {
        (0..PASSES).fold(0.0, |y, _| {
            let p = self.planet(Vec3::new(x, y, z));
            let drawn = drawn(self.view.globe(), p);
            let at = drawn.locate(p);
            y + drawn.mesh_height(at.x, at.z) - at.y
        })
    }

    /// Returns the height in the frame of the highest drawn ground within [`REACH`] of the frame point `(x, z)`:
    /// what a flat piece laid there must clear. The deck of the pad and the test of what is solid read it alike.
    pub fn crest(&self, x: f32, z: f32) -> f32 {
        SPREAD
            .iter()
            .map(|d| self.ground(x + d[0] * REACH, z + d[1] * REACH))
            .fold(self.ground(x, z), f32::max)
    }

    /// Returns the pose in the frame of a piece standing at `spot`: level, with its origin
    /// `clear` metres above the highest ground under its `feet`, given right and back of it.
    pub fn stand(&self, spot: Spot, feet: &[Vec2], clear: f32) -> Affine3A {
        let under = |foot: &Vec2| {
            let at = spot.point(*foot);
            self.ground(at.x, at.y)
        };
        let top = feet.iter().map(under).reduce(f32::max);
        spot.pose(top.map_or(0.0, |top| top + clear))
    }
}
