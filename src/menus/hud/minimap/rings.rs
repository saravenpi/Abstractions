//! The orbit rings of the space view, kept in a layer of their own under the marks.
//!
//! A ring is a circle of many small nodes, and a system has a ring for each planet, so the rings
//! hold most of the nodes of the chart. A planet moves along its ring, which stays where it is, so
//! the rings are spawned again only when one of them grows or shrinks by a pixel at the zoom of
//! the chart, not each time a mark moves.

use bevy::prelude::*;

use super::orbit::Scene;
use crate::computer::{Dot, SIZE, is_moon, ring};

/// Marks the layer that holds the orbit rings of the chart.
#[derive(Component, Debug, Clone, Copy)]
pub struct Orbits;

/// Returns the rows of the planets of `scene` and where each is on the chart.
fn planets(scene: &Scene) -> impl Iterator<Item = Vec2> + '_ {
    let lines = scene.lines.as_slice();
    scene.dots.iter().filter_map(move |(dot, at)| match dot {
        Dot::Row(index) if *index > 0 && !is_moon(lines[*index].target) => Some(*at),
        _ => None,
    })
}

/// Returns what the rings of `scene` depend on: the radius of each ring in map pixels at the
/// zoom of the chart, rounded. The rings are spawned again only when this changes.
pub fn key(scene: &Scene) -> Vec<i32> {
    let pixels = SIZE * scene.lens.zoom;
    planets(scene)
        .map(|at| (at.length() * pixels).round() as i32)
        .collect()
}

/// Spawns the layer of the orbit rings of `scene` on the chart sheet, as large as the sheet so
/// the rings stand where they stood on it.
pub fn spawn(sheet: &mut ChildSpawnerCommands, scene: &Scene) {
    let layer = Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        width: percent(100.0),
        height: percent(100.0),
        ..default()
    };
    sheet.spawn((Orbits, layer)).with_children(|layer| {
        for at in planets(scene) {
            layer.spawn(ring(at.length()));
        }
    });
}
