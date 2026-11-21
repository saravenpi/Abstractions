//! Charts: the ways flat `(x, z)` terrain coordinates map onto the globe.

use bevy::math::{DVec2, DVec3, Vec2};

use super::cube::Face;
use super::tangent::Tangent;

/// Side in chart units of the blocks the origin of a face chart snaps to. It is a whole number of chunks
/// (64), turf periods (256), blade tiles (8) and mesh grid steps (2), so every grid of the chart keeps its
/// lines when the origin moves by whole blocks.
pub const BLOCK: f64 = 3840.0;

/// Smallest cosine between a direction and the middle of a face at which a face chart is rooted near it.
const FACING: f64 = 0.1;

/// A map between flat chart coordinates `(x, z)` and unit directions from the planet centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chart {
    /// The legacy infinite plane: chart coordinates are world coordinates and up is +Y.
    Flat,
    /// One cube face, spanning `-scale` to `scale` on both axes around its middle. Chart coordinates are
    /// 32-bit offsets from `origin`, so they stay small and precise on a planet of any size.
    Face {
        /// Face the chart covers.
        face: Face,
        /// Chart units per face unit: half the width of the face in chart units.
        scale: f64,
        /// Where chart position `(0, 0)` lies, in chart units from the middle of the face: whole blocks.
        origin: DVec2,
    },
    /// A metric chart around one point.
    Tangent(Tangent),
}

impl Chart {
    /// Returns the unit direction at chart position `(x, z)`. The flat chart always gives +Y.
    pub fn dir(&self, x: f32, z: f32) -> DVec3 {
        match self {
            Chart::Flat => DVec3::Y,
            Chart::Face {
                face,
                scale,
                origin,
            } => {
                let at = (*origin + Vec2::new(x, z).as_dvec2()) / *scale;
                face.dir(at.x, at.y)
            }
            Chart::Tangent(tangent) => tangent.dir(Vec2::new(x, z).as_dvec2()),
        }
    }

    /// Returns the chart position of unit direction `dir`. The flat chart always gives zero.
    pub fn locate(&self, dir: DVec3) -> Vec2 {
        match self {
            Chart::Flat => Vec2::ZERO,
            Chart::Face {
                face,
                scale,
                origin,
            } => {
                let (u, v) = face.project(dir);
                (DVec2::new(u, v) * *scale - *origin).as_vec2()
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

    /// Returns where chart position `(0, 0)` lies in chart units from the middle of the chart: the origin of
    /// a face chart, and zero on every other chart.
    pub fn origin(&self) -> DVec2 {
        match self {
            Chart::Face { origin, .. } => *origin,
            _ => DVec2::ZERO,
        }
    }

    /// Returns this chart with chart position `(0, 0)` moved to `origin`, in chart units from the middle of
    /// the face. Only a face chart has an origin to move.
    pub fn rooted(self, origin: DVec2) -> Chart {
        match self {
            Chart::Face { face, scale, .. } => Chart::Face {
                face,
                scale,
                origin,
            },
            chart => chart,
        }
    }

    /// Returns this chart rooted on the block nearest unit direction `dir`, so chart positions around `dir`
    /// are small. A face chart that `dir` does not face, and every other chart, is returned as it is.
    pub fn near(self, dir: DVec3) -> Chart {
        match self {
            Chart::Face { face, scale, .. } if dir.dot(face.axes()[0]) > FACING => {
                let (u, v) = face.project(dir);
                self.rooted((DVec2::new(u, v) * scale / BLOCK).round() * BLOCK)
            }
            chart => chart,
        }
    }

    /// Returns what to add to a chart position of this chart to get the same place on `other`, when the two
    /// are one chart rooted apart, or nothing when they are different charts.
    pub fn carried(&self, other: &Chart) -> Option<DVec2> {
        let same = self.rooted(DVec2::ZERO) == other.rooted(DVec2::ZERO);
        same.then(|| self.origin() - other.origin())
    }

    /// Returns the lowest and the highest chart position the chart covers: the corners of a cube face, and
    /// no bound on a chart without edges.
    pub fn span(&self) -> (Vec2, Vec2) {
        match self {
            Chart::Face { scale, origin, .. } => (
                (DVec2::splat(-*scale) - *origin).as_vec2(),
                (DVec2::splat(*scale) - *origin).as_vec2(),
            ),
            _ => (Vec2::MIN, Vec2::MAX),
        }
    }
}
