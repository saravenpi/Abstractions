//! Finding the ground or sea beside the hatch where the pilot steps out.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Dock;
use crate::space::{Placed, up};

/// Where the player steps out in body axes: beside the cockpit, in front of the wing, where
/// the open canopy is within reach.
const HATCH: DVec3 = DVec3::new(-1.6, 0.0, -1.5);

impl Dock<'_, '_> {
    /// Returns the render position the feet go down on beside the hatch of a ship with pose
    /// `at`: the top of the ground collider there, or the ground the terrain mesh draws while no
    /// collider is loaded yet, which a body stands on as on a collider, or the sea over it.
    pub(super) fn step_out(&self, at: &Placed) -> Option<Vec3> {
        let hatch = at.pos + at.rot * HATCH;
        let render = self.anchor.to_render(hatch);
        let vertical = up(&self.anchor, render);
        let ground = self
            .floor
            .under(render, vertical)
            .or_else(|| self.drawn(render, vertical));
        if self.skies.height(hatch) >= self.skies.sea() {
            return ground;
        }
        let radius = self
            .skies
            .planet()
            .map_or(hatch.length(), |planet| planet.radius);
        let sea = self
            .anchor
            .to_render(hatch.normalize_or(DVec3::Y) * (radius + self.skies.sea()));
        let higher = |feet: Vec3| [sea, feet][usize::from((feet - sea).dot(vertical) > 0.0)];
        Some(ground.map_or(sea, higher))
    }

    /// Returns the render position of the ground the terrain mesh draws under `render`, along
    /// the local up `vertical`, on a planet with ground.
    fn drawn(&self, render: Vec3, vertical: Vec3) -> Option<Vec3> {
        let terrain = self.terrain.as_deref()?;
        if !terrain.globe().has_ground() {
            return None;
        }
        let chart = terrain.locate(self.anchor.to_frame(render));
        let below = chart.y - terrain.mesh_height(chart.x, chart.z);
        Some(render - vertical * below)
    }
}
