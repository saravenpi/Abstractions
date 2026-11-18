//! Street tracing: bending polylines sampled every 2 m that stop where the ground turns bad.

use bevy::prelude::*;

use crate::world::{Surface, Terrain};

/// Distance between street samples.
pub const STEP: f32 = 2.0;

/// Steepest allowed height change between two samples.
const MAX_DROP: f32 = 0.9;

/// Traces a street from `start` heading `angle`, turning `bend` radians per sample, up to `len` meters.
pub fn street(terrain: &Terrain, start: Vec2, angle: f32, bend: f32, len: f32) -> Vec<Vec2> {
    let mut pts = vec![start];
    let mut last = terrain.height(start.x, start.y);
    for i in 0..(len / STEP) as u32 {
        let p = pts[pts.len() - 1] + Vec2::from_angle(angle + bend * i as f32) * STEP;
        let h = terrain.height(p.x, p.y);
        if !walkable(terrain, p) || (h - last).abs() > MAX_DROP {
            break;
        }
        pts.push(p);
        last = h;
    }
    pts
}

/// Returns true when the ground at `p` is dry and soft: never rock, a sea floor or a river bed.
pub fn walkable(terrain: &Terrain, p: Vec2) -> bool {
    terrain.height(p.x, p.y) > 0.8
        && matches!(
            terrain.surface(p.x, p.y),
            Surface::Pond | Surface::Sand | Surface::Grass | Surface::Snow
        )
}

/// Returns the yaw whose local +z points along the world direction `f`.
pub fn yaw_of(f: Vec2) -> f32 {
    f.x.atan2(f.y)
}
