//! Applying spherical dig stamps to chunk height grids and exposed-earth mesh colours.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::DigStamp;
use super::tint::{apply_uv_tint, vertex_depths};
use crate::universe::BodyId;
use crate::world::mesh::{Frame, GRID_POINTS, Span};
use crate::world::{CHUNK_SIZE, Terrain};

/// Maximum cumulative dig depth below original ground height, in metres.
pub const MAX_DIG_DEPTH: f32 = 12.0;

/// Maximum number of crater stamps kept per world so chunk sampling stays bounded.
pub const MAX_DIG_STAMPS: usize = 4096;

/// World-space half-diagonal margin of a chunk used for coarse stamp culling.
pub(super) const CHUNK_CULL_MARGIN: f64 = CHUNK_SIZE as f64 * 0.85;

/// All craters carved by players across every planet of the world.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Digs(Arc<Vec<DigStamp>>);

/// Evaluates cumulative crater depression in metres at unit direction `dir` across `stamps`.
pub(super) fn sample_stamps(stamps: &[&DigStamp], dir: DVec3, globe_radius: f64) -> f32 {
    let mut total = 0.0_f32;
    for stamp in stamps {
        let sd = DVec3::from_array(stamp.dir).normalize_or(DVec3::Y);
        let dist = dir.distance(sd) * globe_radius;
        if dist < f64::from(stamp.radius) && stamp.radius > 0.01 {
            let t = (dist / f64::from(stamp.radius)) as f32;
            let w = 0.5 * (1.0 + (std::f32::consts::PI * t).cos());
            total += w * stamp.depth.max(0.0);
        }
    }
    total.min(MAX_DIG_DEPTH)
}

/// Returns whether a dig stamp is near enough to touch a chunk centred at `center`.
fn stamp_near_chunk(center: DVec3, stamp: &DigStamp, globe_radius: f64) -> bool {
    let sd = DVec3::from_array(stamp.dir).normalize_or(DVec3::Y);
    center.distance(sd) * globe_radius <= CHUNK_CULL_MARGIN + f64::from(stamp.radius.max(0.0))
}

impl Digs {
    /// Appends a new crater stamp to the world list (capped at [`MAX_DIG_STAMPS`]).
    pub fn push(&mut self, stamp: DigStamp) {
        let vec = Arc::make_mut(&mut self.0);
        if vec.len() >= MAX_DIG_STAMPS {
            vec.remove(0);
        }
        vec.push(stamp);
    }

    /// Replaces the full crater list from saved or server state.
    pub fn set(&mut self, stamps: Vec<DigStamp>) {
        self.0 = Arc::new(stamps);
    }

    /// Returns all stored crater stamps.
    pub fn all(&self) -> &[DigStamp] {
        &self.0
    }

    /// Returns total dug depth in metres at planet `planet` and unit direction `dir`.
    pub fn depth_at(&self, planet: BodyId, dir: DVec3, globe_radius: f64) -> f32 {
        let nearby: Vec<&DigStamp> = self
            .0
            .iter()
            .filter(|s| {
                s.planet == planet
                    && dir.distance(DVec3::from_array(s.dir).normalize_or(DVec3::Y)) * globe_radius
                        <= f64::from(s.radius)
            })
            .collect();
        if nearby.is_empty() {
            0.0
        } else {
            sample_stamps(&nearby, dir, globe_radius)
        }
    }

    /// Returns true if `stamp` overlaps chunk `coord` on `terrain`.
    pub fn overlaps_chunk(&self, terrain: &Terrain, coord: IVec2, stamp: &DigStamp) -> bool {
        let center = Span::leaf(terrain, coord, 2).dir(1, 1);
        stamp_near_chunk(center, stamp, terrain.globe().radius())
    }

    /// Subtracts active craters from `heights`, returning true if any stamp affected the chunk.
    pub fn apply_to_grid(
        &self,
        terrain: &Terrain,
        coord: IVec2,
        planet: BodyId,
        heights: &mut [Vec<f32>],
    ) -> bool {
        if self.0.is_empty() || heights.len() != GRID_POINTS {
            return false;
        }
        let globe_radius = terrain.globe().radius();
        let center = Span::leaf(terrain, coord, 2).dir(1, 1);
        let active: Vec<&DigStamp> = self
            .0
            .iter()
            .filter(|s| s.planet == planet && stamp_near_chunk(center, s, globe_radius))
            .collect();
        if active.is_empty() {
            return false;
        }
        let span = Span::leaf(terrain, coord, GRID_POINTS - 1);
        let mut touched = false;
        for (i, col) in heights.iter_mut().enumerate().take(GRID_POINTS) {
            for (j, h) in col.iter_mut().enumerate().take(GRID_POINTS) {
                let d = sample_stamps(&active, span.dir(i, j), globe_radius);
                if d > 0.0 {
                    *h -= d;
                    touched = true;
                }
            }
        }
        touched
    }

    /// Darkens triangle facets of `mesh` whose vertices lie inside dug craters on `planet`.
    pub fn tint_mesh(&self, mesh: &mut Mesh, planet: BodyId, frame: &Frame, globe_radius: f64) {
        if let Some(depths) = vertex_depths(&self.0, mesh, planet, frame, globe_radius) {
            apply_uv_tint(mesh, &depths);
        }
    }
}
