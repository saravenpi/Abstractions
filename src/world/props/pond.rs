//! Pond plants: pads, flowers and specks on the water surface, and stalks that reach down to the pond floor.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::world::propmesh::Plant;
use crate::world::scatter::Placement;
use crate::world::shapes::{FlatMesh, shade};
use crate::world::solids::{fan, frustum, icosa};

use self::parts::{FLOAT, bloom, both, pad, stalk};

mod parts;

/// Returns an upper bound of the triangles a pond plant adds.
pub fn pond_tris(plant: Plant) -> usize {
    match plant {
        Plant::Lily => 32,
        Plant::Lotus => 40,
        Plant::Cattail => 32,
        _ => 12,
    }
}

/// Adds a cupped lotus pad and a tall stalk ending in a white flower, a pink flower or a seed pod.
fn lotus(m: &mut FlatMesh, p: &Placement, tone: f32) {
    let floor = p.pos;
    let top = Vec3::new(floor.x, FLOAT, floor.z);
    let green = shade([0.2, 0.5, 0.22], tone);
    stalk(m, [floor, top], p.yaw, green);
    pad(
        m,
        top,
        [0.7 * p.scale, 0.07],
        shade([0.18, 0.46, 0.2], tone),
    );
    let foot = floor + Vec3::new(0.4, 0.0, 0.0);
    let head = Vec3::new(foot.x, FLOAT + 0.3 + 0.3 * p.yaw / TAU, foot.z);
    stalk(m, [foot, head], p.yaw, green);
    match p.variant {
        0 if p.yaw > 4.2 => {
            let pod = shade([0.45, 0.5, 0.2], tone);
            icosa(m, head, Vec3::new(0.08, 0.05, 0.08), (0.1, 5), pod);
        }
        0 => {}
        v => bloom(m, head, p.yaw, v == 2),
    }
}

/// Adds a lily pad, one in three with a flower.
fn lily(m: &mut FlatMesh, p: &Placement, tone: f32) {
    let top = Vec3::new(p.pos.x, FLOAT, p.pos.z);
    let green = shade([0.2, 0.5, 0.22], tone);
    stalk(m, [p.pos, top], p.yaw, green);
    pad(m, top, [0.5 * p.scale, 0.0], green);
    if p.variant == 2 {
        let petals = [6.0, 0.1, 0.1, 0.0, 0.09];
        fan(
            m,
            top + Vec3::Y * FLOAT,
            petals,
            p.yaw,
            shade([0.97, 0.95, 0.9], 1.0),
        );
    }
}

/// Adds a cattail with a brown head and a bent leaf.
fn cattail(m: &mut FlatMesh, p: &Placement, tone: f32) {
    let floor = p.pos;
    let green = shade([0.2, 0.5, 0.22], tone);
    let head = Vec3::new(floor.x, 0.9 + 0.7 * p.yaw / TAU, floor.z);
    stalk(m, [floor, head], p.yaw, green);
    stalk(
        m,
        [floor, head + Vec3::new(0.3, -0.5, 0.1)],
        p.yaw + 1.0,
        green,
    );
    let seed = Vec2::new(0.05, 0.045);
    frustum(
        m,
        head - Vec3::Y * 0.3,
        head - Vec3::Y * 0.05,
        seed,
        shade([0.4, 0.25, 0.12], tone),
    );
}

/// Adds three specks of duckweed on the water.
fn duckweed(m: &mut FlatMesh, p: &Placement, tone: f32) {
    let speck = shade([0.4, 0.7, 0.2], tone);
    for o in [(0.0, 0.0), (0.15, 0.08), (-0.1, 0.14)] {
        let c = Vec3::new(p.pos.x + o.0, FLOAT, p.pos.z + o.1);
        both(
            m,
            [
                c,
                c + Vec3::new(0.06, 0.0, 0.0),
                c + Vec3::new(0.0, 0.0, 0.06),
            ],
            speck,
        );
    }
}

/// Adds the pond plant of placement `p`; the water level is `y = 0` and `p.pos.y` is the pond floor.
pub fn pond(m: &mut FlatMesh, p: &Placement, tone: f32) {
    match p.plant {
        Plant::Lily => lily(m, p, tone),
        Plant::Lotus => lotus(m, p, tone),
        Plant::Cattail => cattail(m, p, tone),
        _ => duckweed(m, p, tone),
    }
}
