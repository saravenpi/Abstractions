//! The drawn ground as the camera sees it: where no collider is loaded yet, the camera still
//! stops on the ground the terrain mesh draws, so it never looks from under the planet.

use bevy::prelude::*;

use crate::space::Anchor;
use crate::world::Terrain;

/// Samples along a cast tested against the drawn ground.
const STEPS: usize = 12;

/// Returns how far a sphere of `radius` travels from `origin` along the unit `direction` before
/// it reaches the ground `terrain` draws, within `max` metres, or `None` when it stays above it.
pub fn drawn(
    (terrain, anchor): (&Terrain, &Anchor),
    (origin, direction): (Vec3, Vec3),
    max: f32,
    radius: f32,
) -> Option<f32> {
    if !terrain.globe().has_ground() {
        return None;
    }
    let under = |distance: f32| {
        let chart = terrain.locate(anchor.to_frame(origin + direction * distance));
        chart.y - radius < terrain.mesh_height(chart.x, chart.z)
    };
    if under(0.0) {
        return None;
    }
    (1..=STEPS)
        .map(|step| max * step as f32 / STEPS as f32)
        .find(|distance| under(*distance))
        .map(|distance| (distance - max / STEPS as f32).max(0.0))
}
