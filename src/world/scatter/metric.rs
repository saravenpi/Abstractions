//! The true-metre measure of one chunk: chart units are not metres on a globe, and shrink toward cube edges.

use bevy::math::{DMat2, DVec2, DVec3};
use bevy::prelude::*;

use crate::world::globe::Chart;
use crate::world::{CHUNK_SIZE, Terrain};

/// How the chart of a terrain view measures the ground of one chunk.
///
/// The chart is taken as linear over the chunk, which holds within two percent on the smallest planet. Distances
/// between props do not use it: they are measured between embedded positions.
pub struct Metric {
    /// Position of the chunk center on the sea level sphere, in metres from the planet centre. The legacy plane
    /// has none: its chart is already metric.
    origin: Option<DVec3>,
    /// Chart offsets of one metre along each of two perpendicular directions on the ground.
    back: Mat2,
    /// True area of the chunk over the area of a square of `CHUNK_SIZE` metres.
    share: f32,
}

impl Metric {
    /// Returns the measure of the chunk centred on chart position `center` of `terrain`.
    pub fn of(terrain: &Terrain, center: Vec2) -> Self {
        if terrain.chart() == Chart::Flat {
            return Self {
                origin: None,
                back: Mat2::IDENTITY,
                share: 1.0,
            };
        }
        let half = CHUNK_SIZE * 0.5;
        let at = |o: Vec2| terrain.embed(center.x + o.x, 0.0, center.y + o.y);
        let span = |o: Vec2| (at(o) - at(-o)) / f64::from(CHUNK_SIZE);
        let (x, z) = (span(Vec2::X * half), span(Vec2::Y * half));
        let east = x.normalize();
        let south = east.cross(terrain.up(center.x, center.y));
        let ground = DMat2::from_cols(
            DVec2::new(x.dot(east), x.dot(south)),
            DVec2::new(z.dot(east), z.dot(south)),
        );
        Self {
            origin: Some(at(Vec2::ZERO)),
            back: ground.inverse().as_mat2(),
            share: ground.determinant().abs() as f32,
        }
    }

    /// Returns `count`, a number per square of `CHUNK_SIZE` metres, scaled to the true area of the chunk.
    pub fn scaled(&self, count: usize) -> usize {
        (count as f32 * self.share).round() as usize
    }

    /// Returns the chart offset of a ground offset in metres.
    pub fn chart(&self, metres: Vec2) -> Vec2 {
        self.back * metres
    }

    /// Returns the true slope, rise over run, of ground whose height changes by `grade` per chart unit.
    pub fn slope(&self, grade: Vec2) -> f32 {
        (self.back.transpose() * grade).length()
    }

    /// Returns where chart position `w`, at chunk offset `local`, lies on the sea level sphere, in metres
    /// from the chunk center. Distances between two such points are true ground distances.
    pub fn spot(&self, terrain: &Terrain, w: Vec2, local: Vec2) -> Vec3 {
        match self.origin {
            Some(origin) => (terrain.embed(w.x, 0.0, w.y) - origin).as_vec3(),
            None => Vec3::new(local.x, 0.0, local.y),
        }
    }
}
