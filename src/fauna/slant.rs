//! The chart of a planet around one point of a flat frame, as a plane, so a ray of sight finds the
//! chart position of each of its points without locating each one on the globe.

use bevy::math::{Vec2, Vec3};

use super::place::Spot;
use crate::world::Terrain;

/// Metres along each axis of the frame at which a [`Slant`] measures the chart.
const SPAN: f32 = 64.0;

/// Where a point of a frame lies in the chart of a planet and how the chart moves along each axis of
/// the frame. It holds within the reach of sight of the point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slant {
    /// The point, `(x, z)` in the frame.
    at: Vec2,
    /// Its chart position.
    chart: Vec2,
    /// Chart metres per metre along the frame's x.
    x: Vec2,
    /// Chart metres per metre along the frame's z.
    z: Vec2,
}

impl Slant {
    /// Returns the chart position of `(x, z)` of the frame, near the point of the slant.
    fn chart(&self, at: Vec2) -> Vec2 {
        let d = at - self.at;
        self.chart + self.x * d.x + self.z * d.y
    }
}

impl Spot {
    /// Returns the chart position `(x, z)` of `terrain` of the point over `(x, z)` of the frame.
    pub fn chart(&self, terrain: &Terrain, at: Vec2) -> Vec2 {
        let pos = self.planet(Vec3::new(at.x, 0.0, at.y));
        let chart = terrain.locate(pos.normalize() * terrain.globe().radius());
        Vec2::new(chart.x, chart.z)
    }

    /// Returns the slant of the chart of `terrain` around the point over `(x, z)` of the frame.
    pub fn slant(&self, terrain: &Terrain, at: Vec2) -> Slant {
        let chart = self.chart(terrain, at);
        let along = |step: Vec2| (self.chart(terrain, at + step) - chart) / SPAN;
        Slant {
            at,
            chart,
            x: along(Vec2::X * SPAN),
            z: along(Vec2::Y * SPAN),
        }
    }

    /// Returns what [`Spot::ground_by`] returns for the point `at` within sight of the point of
    /// `slant`, its chart position read off the slant instead of located on the globe.
    pub fn ground_near(
        &self,
        terrain: &Terrain,
        (slant, at): (&Slant, Vec2),
        height: impl Fn(f32, f32) -> f32,
    ) -> f32 {
        let chart = slant.chart(at);
        self.rise(
            at,
            terrain.globe().radius() + f64::from(height(chart.x, chart.y)),
        )
    }
}
