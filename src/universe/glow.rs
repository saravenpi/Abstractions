//! The light of a galaxy as the sky draws it: the stars it holds and the dust that dims them at
//! any place of it, read from the same model that places its stars.

use bevy::math::DVec3;

use super::GLY;
use super::density::Parts;
use super::galaxy::Shape;
use super::home::HOME_GALAXY;
use super::spec::{GalaxyKind, GalaxySpec};

/// How far the stars of a galaxy reach, in radii of its disc: past it the density is nothing.
const REACH: f64 = 1.4;

/// Half thickness of the slab that holds the light of a spiral, in standard deviations of its
/// disc, and of its bulge: where the density has fallen below a thousandth of its peak.
const SLAB: (f64, f64) = (4.0, 3.5);

/// Share of its radius a ball or an irregular cloud holds its light within, above and below.
const ROUND: f64 = 0.8;

/// The star density and the dust of one galaxy, ready to be read at many places.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glow {
    shape: Shape,
}

impl Glow {
    /// Returns the glow of `galaxy`.
    pub fn of(galaxy: &GalaxySpec) -> Self {
        Self {
            shape: Shape::new(galaxy.seed, galaxy.id == HOME_GALAXY),
        }
    }

    /// Returns the expected stars per sector at `pos` metres from the galaxy centre, split by
    /// the structure they belong to, and the dust there as the optical depth per game
    /// light-year.
    pub fn at(&self, pos: DVec3) -> (Parts, f64) {
        let place = self.shape.place(pos / GLY);
        (self.shape.parts_at(&place), self.shape.dust_at(&place))
    }

    /// Returns the radius in metres of the disc of the galaxy.
    pub fn radius(&self) -> f64 {
        self.shape.radius * GLY
    }

    /// Returns the radius in metres of the sphere that holds all the light of the galaxy.
    pub fn reach(&self) -> f64 {
        REACH * self.shape.radius * GLY
    }

    /// Returns the half thickness in metres of the slab, centred on the plane of the disc, that
    /// holds all the light of the galaxy.
    pub fn slab(&self) -> f64 {
        let shape = &self.shape;
        let disc = SLAB.0
            * shape.thick
            * if shape.kind == GalaxyKind::Irregular {
                4.0
            } else {
                1.0
            };
        let share = match shape.kind {
            GalaxyKind::Spiral => disc.max(SLAB.1 * shape.core),
            GalaxyKind::Elliptical => ROUND,
            GalaxyKind::Irregular => disc.max(SLAB.1 * shape.core).min(ROUND),
        };
        share * shape.radius * GLY
    }
}
