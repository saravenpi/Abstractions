//! Soil: the ground of the planet as the terrain meshes draw it.
//!
//! Near the robot the ground has colliders, farther it has none, and a collider is not quite the
//! triangles the player sees: a javelin reads the drawn ground, the grid of the mesh shown there,
//! so it never sinks into the picture of the ground.

use bevy::prelude::*;

use super::ground::{Ground, Soil};
use super::tune::SLOPE;
use crate::space::Anchor;
use crate::world::{Chunks, Drawn, Terrain};

/// The drawn ground of the planet the robot is on.
pub struct Planet<'a> {
    /// Where render space sits on the planet.
    pub anchor: &'a Anchor,
    /// The height field.
    pub terrain: &'a Terrain,
    /// The meshes drawn, when the world has them.
    pub chunks: Option<&'a Chunks>,
}

impl<'a> Planet<'a> {
    /// Returns the drawn ground of `terrain` seen from `anchor`, on the meshes of `chunks`.
    pub fn new(anchor: &'a Anchor, terrain: &'a Terrain, chunks: Option<&'a Chunks>) -> Self {
        Self {
            anchor,
            terrain,
            chunks,
        }
    }

    /// Returns the chart position and height of the render position `at`.
    fn chart(&self, at: Vec3) -> Vec3 {
        self.terrain.locate(self.anchor.to_frame(at))
    }

    /// Returns the render position of the drawn ground at chart position `at`.
    fn on(&self, at: Vec2) -> Vec3 {
        let grid = self.chunks.and_then(|c| c.drawn_at(self.terrain, at));
        let height = grid.unwrap_or(Drawn::FINE).height(self.terrain, at);
        self.anchor
            .to_render(self.terrain.embed(at.x, height, at.y))
    }
}

impl Ground for Planet<'_> {
    /// Returns the height of `at` over the drawn ground.
    fn clearance(&self, at: Vec3) -> f32 {
        let chart = self.chart(at);
        let at = Vec2::new(chart.x, chart.z);
        let grid = self.chunks.and_then(|c| c.drawn_at(self.terrain, at));
        chart.y - grid.unwrap_or(Drawn::FINE).height(self.terrain, at)
    }

    /// Returns the drawn ground straight over or under `at`.
    fn lift(&self, at: Vec3) -> Vec3 {
        let chart = self.chart(at);
        self.on(Vec2::new(chart.x, chart.z))
    }

    /// Returns the normal of the drawn triangles under `at`, turned to the local up.
    fn normal(&self, at: Vec3) -> Vec3 {
        let chart = self.chart(at);
        let at = Vec2::new(chart.x, chart.z);
        let o = self.on(at);
        let (x, z) = (
            self.on(at + Vec2::X * SLOPE) - o,
            self.on(at + Vec2::Y * SLOPE) - o,
        );
        let up = (self.anchor.rotation * self.terrain.up(at.x, at.y)).as_vec3();
        let normal = z.cross(x).normalize_or(up);
        normal * normal.dot(up).signum()
    }

    /// Returns what covers the ground under `at`.
    fn soil(&self, at: Vec3) -> Soil {
        let chart = self.chart(at);
        Soil::of(self.terrain.surface(chart.x, chart.z))
    }
}
