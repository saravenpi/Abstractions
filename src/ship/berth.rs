//! Where the ship starts: a dry, flat spot near the spawn point, on its gear.

use std::f64::consts::TAU;

use bevy::math::{DQuat, DVec3};

use super::body::Ship;
use super::landing::GEAR_DROP;
use crate::space::Frame;
use crate::universe::BodyId;

/// Distance in metres from the spawn point to the first ring of candidate spots, and between rings.
const RING: (f64, f64) = (14.0, 8.0);

/// Rings searched and spots tried on each.
const TRIES: (u32, u32) = (10, 12);

/// Lowest ground in metres above the sea a spot may have.
const DRY: f64 = 1.5;

/// Distance in metres from the centre of a spot to the four points that must be level with it.
const FOOTPRINT: f64 = 4.0;

/// Largest height difference in metres between the centre of a spot and its footprint.
const LEVEL: f64 = 0.5;

/// Returns two unit directions tangent to the globe at `up`, at right angles to each other.
fn tangents(up: DVec3) -> (DVec3, DVec3) {
    let east = up.any_orthonormal_vector();
    (east, up.cross(east))
}

/// Returns true when the ground at `at` is dry and level over the footprint of the ship. The
/// sea stands at `sea` metres, lower than any ground on a planet without one.
fn dry_and_flat(height: &dyn Fn(DVec3) -> f64, at: DVec3, sea: f64) -> bool {
    let (east, north) = tangents(at.normalize());
    let centre = height(at);
    let level = |axis: DVec3| (height(at + axis * FOOTPRINT) - centre).abs() < LEVEL;
    centre > sea + DRY && [east, -east, north, -north].into_iter().all(level)
}

/// Returns the spot where the ship starts near the point `near` of a planet, as a point on the
/// same sphere: the first dry, flat spot on rings around it, or a point beside it when none is.
///
/// `height` gives the terrain height in metres above sea level at a position in the planet
/// frame, and `sea` the height of the sea: minus infinity on a dry planet, where a basin below
/// sea level is ground like any other.
pub fn landing_spot(height: &dyn Fn(DVec3) -> f64, near: DVec3, sea: f64) -> DVec3 {
    let (radius, (east, north)) = (near.length(), tangents(near.normalize()));
    let spot = |i: u32| {
        let (ring, turn) = (f64::from(i / TRIES.1), f64::from(i % TRIES.1));
        let angle = TAU * (turn + 0.5 * ring) / f64::from(TRIES.1);
        let reach = RING.0 + RING.1 * ring;
        (near + (east * angle.cos() + north * angle.sin()) * reach).normalize() * radius
    };
    let found = (0..TRIES.0 * TRIES.1)
        .map(spot)
        .find(|at| dry_and_flat(height, *at, sea));
    found.unwrap_or_else(|| spot(0))
}

/// Returns a ship landed on its gear at the point `at` of planet `body`, level with the globe:
/// on the ground, or on the sea of height `sea` where the ground is under it.
pub fn parked(body: BodyId, ground: (&dyn Fn(DVec3) -> f64, f64), at: DVec3, radius: f64) -> Ship {
    let (height, sea) = ground;
    let up = at.normalize();
    let pos = up * (radius + height(at).max(sea) + GEAR_DROP);
    let mut ship = Ship::at_rest(
        Frame::Surface(body),
        pos,
        DQuat::from_rotation_arc(DVec3::Y, up),
    );
    (ship.landed, ship.touching) = (true, true);
    ship
}
