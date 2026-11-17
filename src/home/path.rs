//! The path: flat slabs from the pad round the end of the container to the hatch and the station.

use bevy::prelude::*;

use super::container::DOOR;
use super::draft::Draft;
use super::paint::DECK;
use super::tune::{CONTAINER, PAD, PAD_HALF, STATION, Spot};

/// Length and width of a slab in metres.
const SLAB: Vec2 = Vec2::new(1.1, 0.8);

/// Distance in metres between the centres of two slabs.
const PACE: f32 = 1.7;

/// Height in metres of a slab above the highest ground under it, and its depth below the lowest.
const SET: (f32, f32) = (0.04, 0.15);

/// Distance in metres from the pad centre to the first slab.
const START: f32 = PAD_HALF + 4.0;

/// Adds one level slab centred `at`, east and south of the base centre, turned by `yaw`.
fn slab(draft: &mut Draft, at: Vec2, yaw: f32) {
    let half = SLAB * 0.5;
    let corners =
        [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(x, z)| Vec2::new(x, z) * half);
    draft.stand(Spot { at, yaw }, &corners, SET.0);
    let low = draft.lowest(&corners) - SET.1;
    draft.deck(
        Vec3::new(-half.x, low, -half.y),
        Vec3::new(half.x, 0.0, half.y),
        DECK,
    );
}

/// Adds slabs from `from` up to, but not on, `to`.
fn run(draft: &mut Draft, from: Vec2, to: Vec2) {
    let way = to - from;
    let count = (way.length() / PACE).round().max(1.0) as u32;
    let yaw = (-way.y).atan2(way.x);
    for k in 0..count {
        slab(draft, from + way * (k as f32 / count as f32), yaw);
    }
}

/// Draws the path: from the pad to the free end of the container, along its front to the
/// hatch, and on to the station.
pub fn build(draft: &mut Draft) {
    let corner = CONTAINER.point(Vec2::new(4.4, 2.6));
    let hatch = CONTAINER.point(Vec2::new(DOOR.x, 3.0));
    let dock = STATION.point(Vec2::new(0.4, 2.6));
    let start = PAD.at + (corner - PAD.at).normalize() * START;
    run(draft, start, corner);
    run(draft, corner, hatch);
    run(draft, hatch, dock);
}
