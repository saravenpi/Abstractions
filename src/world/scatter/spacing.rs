//! How far apart big props stand and how each land prop sits on the visible ground.

use bevy::prelude::*;

use super::Placement;
use crate::world::propmesh::{Plant, TREES};
use crate::world::{SEA_LEVEL, Terrain};

/// Room a tree of scale one keeps around its trunk, in meters.
const TREE_ROOM: f32 = 1.1;

/// Room a bush, rock, log or wreck of scale one keeps around its center, in meters.
const LOW_ROOM: f32 = 0.6;

/// Step of the finite difference that measures the slope under a rock, in meters.
const SLOPE_STEP: f32 = 1.0;

/// How far a rock of scale one sinks per unit of slope, about the radius of its rim.
const RIM: f32 = 0.9;

/// Deepest a rock of scale one sinks into a slope, in meters.
const SINK_CAP: f32 = 0.5;

/// Returns the radius of the circle a big prop keeps free of other big props.
pub fn room(p: &Placement) -> f32 {
    let base = if (p.plant as usize) < TREES {
        TREE_ROOM
    } else {
        LOW_ROOM
    };
    base * p.scale
}

/// Returns true when `p` stands closer to an already placed big prop than their two rooms allow.
pub fn crowded(placed: &[Placement], p: &Placement) -> bool {
    placed
        .iter()
        .any(|q| q.pos.xz().distance(p.pos.xz()) < room(q) + room(p))
}

/// Returns how far a rock of scale `scale` at `w` sinks so that its downhill rim meets the ground.
fn sink(terrain: &Terrain, w: Vec2, scale: f32) -> f32 {
    let d = |o: Vec2| terrain.height(w.x + o.x, w.y + o.y) - terrain.height(w.x - o.x, w.y - o.y);
    let slope =
        Vec2::new(d(Vec2::X * SLOPE_STEP), d(Vec2::Y * SLOPE_STEP)).length() / (2.0 * SLOPE_STEP);
    (slope * RIM).min(SINK_CAP) * scale
}

/// Returns the height a prop stands at: the visible terrain mesh on land, with rocks sunk into slopes, and the
/// analytic floor `height` under water.
pub fn ground(terrain: &Terrain, w: Vec2, p: (Plant, f32), height: f32) -> f32 {
    let (plant, scale) = p;
    if height < SEA_LEVEL {
        return height;
    }
    let y = terrain.mesh_height(w.x, w.y);
    if matches!(plant, Plant::Rock | Plant::IceRock | Plant::Sandstone) {
        y - sink(terrain, w, scale)
    } else {
        y
    }
}
