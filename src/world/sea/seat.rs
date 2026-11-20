//! The seat of the sea: the metric chart, fixed to the planet, that the mesh, the waves and the
//! floor map are laid out in.
//!
//! It does not depend on the render anchor or on the cube face of the shared terrain view, so a
//! rebase or a face change moves nothing in it. It is only replaced when the focus strays far
//! from its middle or reaches another planet.

use std::sync::Arc;

use bevy::math::{DMat3, DQuat, DVec2, DVec3};
use bevy::prelude::*;

use super::bed::Bed;
use super::depth::{TEXEL, encode};
use crate::world::globe::{Chart, Face, Tangent};
use crate::world::{Globe, SEA_LEVEL, Sited, Terrain};

/// Height in meters around sea level inside which a texel reads the terrain mesh instead of
/// the smooth ground, so the map waterline is the one the eye sees.
const SHORE_BAND: f32 = 1.5;

/// The chart of the sea around one point of a planet.
#[derive(Component, Clone)]
pub struct Seat {
    /// Metric view of the planet centred on the seat. Its `(x, z)` are the sea coordinates.
    view: Terrain,
    /// Views of the six cube faces, whose 2 m grids are the finest terrain mesh.
    faces: [Terrain; 6],
}

impl Seat {
    /// Returns the seat centred under planet position `at` of `globe`, with chart x along the
    /// parallel through it.
    pub fn at(globe: &Arc<Globe>, at: DVec3) -> Self {
        let up = at.normalize_or(DVec3::Y);
        let east = DVec3::Y.cross(up);
        let east = if east.length() < 0.1 { DVec3::X } else { east };
        let tangent = Tangent::new(up, east, globe.radius(), DVec2::ZERO);
        Self {
            view: Terrain::on(globe.clone(), Chart::Tangent(tangent)),
            faces: Face::ALL
                .map(|face| Terrain::on(globe.clone(), globe.face_chart(face).near(up))),
        }
    }

    /// Returns true when the seat lies on `globe` itself, not on another planet.
    pub fn holds(&self, globe: &Arc<Globe>) -> bool {
        Arc::ptr_eq(self.view.globe(), globe)
    }

    /// Returns the sea level radius of the planet in meters.
    pub fn radius(&self) -> f64 {
        self.view.globe().radius()
    }

    /// Returns the sea coordinates of planet position `at`.
    pub fn coords(&self, at: DVec3) -> Vec2 {
        let chart = self.view.locate(at);
        Vec2::new(chart.x, chart.z)
    }

    /// Returns true when sea coordinates `coords` are too far from the middle to keep the seat:
    /// past a third of the radius, held between 500 m and 4 km. That is far enough to be rare
    /// and near enough for 32-bit coordinates to stay within a millimeter.
    pub fn strayed(&self, coords: Vec2) -> bool {
        let leash = (self.view.globe().radius() / 3.0).clamp(500.0, 4000.0);
        f64::from(coords.length()) > leash
    }

    /// Returns the pose of the seat on the planet: the origin at sea level in its middle, +X
    /// along chart x, +Y up and +Z along chart z.
    pub fn sited(&self) -> Sited {
        let up = self.view.up(0.0, 0.0);
        let east = self.view.chart().east(0.0, 0.0);
        Sited {
            pos: self.view.embed(0.0, SEA_LEVEL, 0.0),
            rot: DQuat::from_mat3(&DMat3::from_cols(east, up, east.cross(up))),
        }
    }

    /// Returns the height the finest terrain mesh draws in direction `dir`.
    fn mesh_height(&self, dir: DVec3) -> f32 {
        let holder = Face::locate(dir).0;
        let face = Face::ALL.iter().position(|face| *face == holder);
        let view = &self.faces[face.unwrap_or_default()];
        let at = view.chart().locate(dir);
        view.mesh_height(at.x, at.y)
    }
}

impl Bed for Seat {
    /// Reads the bed at sea coordinates `texel * TEXEL`: the smooth ground far from the
    /// waterline and the terrain mesh near it.
    fn texel(&self, texel: IVec2) -> [u8; 4] {
        let at = texel.as_vec2() * TEXEL;
        let dir = self.view.up(at.x, at.y);
        let globe = self.view.globe();
        let smooth = globe.height(dir);
        let height = match smooth.abs() < SHORE_BAND {
            true => self.mesh_height(dir),
            false => smooth,
        };
        encode(SEA_LEVEL - height, globe.pond_at(dir))
    }
}
