//! Charts: the ways flat `(x, z)` terrain coordinates map onto the globe.

use bevy::math::{DVec3, Vec2};

use super::cube::Face;
use super::tangent::Tangent;

/// A map between flat chart coordinates `(x, z)` and unit directions from the planet centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chart {
    /// The legacy infinite plane: chart coordinates are world coordinates and up is +Y.
    Flat,
    /// One cube face, spanning `-scale` to `scale` on both axes.
    Face {
        /// Face the chart covers.
        face: Face,
        /// Chart units per face unit: half the width of the face in chart units.
        scale: f64,
    },
    /// A metric chart around one point.
    Tangent(Tangent),
}

impl Chart {
    /// Returns the unit direction at chart position `(x, z)`. The flat chart always gives +Y.
    pub fn dir(&self, x: f32, z: f32) -> DVec3 {
        match self {
            Chart::Flat => DVec3::Y,
            Chart::Face { face, scale } => face.dir(f64::from(x) / scale, f64::from(z) / scale),
            Chart::Tangent(tangent) => tangent.dir(Vec2::new(x, z).as_dvec2()),
        }
    }

    /// Returns the chart position of unit direction `dir`. The flat chart always gives zero.
    pub fn locate(&self, dir: DVec3) -> Vec2 {
        match self {
            Chart::Flat => Vec2::ZERO,
            Chart::Face { face, scale } => {
                let (u, v) = face.project(dir);
                Vec2::new((u * scale) as f32, (v * scale) as f32)
            }
            Chart::Tangent(tangent) => tangent.locate(dir).as_vec2(),
        }
    }

    /// Returns the unit direction, tangent to the globe, in which chart x grows at `(x, z)`.
    pub fn east(&self, x: f32, z: f32) -> DVec3 {
        if matches!(self, Chart::Flat) {
            return DVec3::X;
        }
        let here = self.dir(x, z);
        let step = self.dir(x + 1.0, z) - here;
        (step - here * step.dot(here)).normalize()
    }
}
