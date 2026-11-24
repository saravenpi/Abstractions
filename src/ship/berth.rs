//! Where the ship starts: a dry, flat spot near the spawn point, on its gear.

use std::f64::consts::TAU;

use bevy::math::{DMat3, DQuat, DVec3};

use super::body::Ship;
use super::landing::GEAR_DROP;
use crate::home::Base;
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

/// Minimum horizontal separation in metres between berthed ships around the home base.
const BAY_GAP: f64 = 18.0;

/// Minimum distance in metres from another ship before slot 0 keeps the landing pad.
const PAD_CLEAR: f64 = 16.0;

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

/// Returns a landed rotation at unit normal `up` with the nose pointed along `base.heading`.
fn pad_facing(base: &Base, up: DVec3) -> DQuat {
    let raw = base.heading - up * base.heading.dot(up);
    let fwd = raw.try_normalize().unwrap_or_else(|| tangents(up).0);
    let back = -fwd;
    DQuat::from_mat3(&DMat3::from_cols(up.cross(back), up, back))
}

/// Scans candidate bay rings around `base.pad` and returns the `target`-th valid bay outside the
/// base and separated from `chosen` by [`BAY_GAP`], advancing `found` and `chosen` as bays qualify.
fn pick_bay(
    base: &Base,
    height: &dyn Fn(DVec3) -> f64,
    (radius, sea, strict): (f64, f64, bool),
    (target, found): (u32, &mut u32),
    chosen: &mut Vec<DVec3>,
) -> Option<DVec3> {
    let up = base.pad.normalize();
    let centre = up * radius;
    let fwd = (base.heading - up * base.heading.dot(up))
        .try_normalize()
        .unwrap_or_else(|| tangents(up).0);
    let side = up.cross(fwd).normalize();
    for r_idx in 0..TRIES.0 {
        let reach = 24.0 + 16.0 * f64::from(r_idx);
        for a_idx in 0..TRIES.1 {
            let angle = TAU * (f64::from(a_idx) + 0.5 * f64::from(r_idx)) / f64::from(TRIES.1);
            let cand =
                (centre + (side * angle.cos() + fwd * angle.sin()) * reach).normalize() * radius;
            let clear =
                !base.covers(cand, 6.0) && chosen.iter().all(|o| o.distance(cand) >= BAY_GAP);
            let ok = if strict {
                dry_and_flat(height, cand, sea)
            } else {
                height(cand) > sea + DRY
            };
            if !clear || !ok {
                continue;
            }
            if *found == target {
                return Some(cand);
            }
            *found += 1;
            chosen.push(cand);
        }
    }
    None
}

/// Returns the surface position and orientation for player `slot` around `base`: slot 0 takes the
/// landing pad unless already taken by `occupied`, and later slots pick distinct dry flat bays
/// outside the base footprint without overlapping any ship in `occupied`.
pub fn berth_bay(
    base: &Base,
    height: &dyn Fn(DVec3) -> f64,
    (radius, sea): (f64, f64),
    slot: u32,
    occupied: &[DVec3],
) -> (DVec3, DQuat) {
    let pad_up = base.pad.normalize();
    let on_pad = |o: &DVec3| (o.normalize() * radius).distance(pad_up * radius) < PAD_CLEAR;
    if slot == 0 && !occupied.iter().any(on_pad) {
        return (base.pad + pad_up * GEAR_DROP, base.pad_rot());
    }
    let target = slot.saturating_sub(1);
    let mut chosen: Vec<DVec3> = std::iter::once(pad_up * radius)
        .chain(occupied.iter().map(|o| o.normalize() * radius))
        .collect();
    let mut found = 0u32;
    let cand = pick_bay(
        base,
        height,
        (radius, sea, true),
        (target, &mut found),
        &mut chosen,
    )
    .or_else(|| {
        pick_bay(
            base,
            height,
            (radius, sea, false),
            (target, &mut found),
            &mut chosen,
        )
    })
    .unwrap_or_else(|| {
        let angle = TAU * f64::from(slot % TRIES.1) / f64::from(TRIES.1);
        let reach = 40.0 + BAY_GAP * f64::from(slot / TRIES.1);
        (pad_up * radius
            + (tangents(pad_up).0 * angle.cos() + tangents(pad_up).1 * angle.sin()) * reach)
            .normalize()
            * radius
    });
    let up = cand.normalize();
    (
        up * (radius + height(cand).max(sea) + GEAR_DROP),
        pad_facing(base, up),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Multiplayer berth slots give slot 0 the pad and place later ships in non-overlapping bays.
    #[test]
    fn berth_bays_separate_multiplayer_ships_and_respect_occupied_pad() {
        let body = BodyId::default();
        let radius = 6000.0;
        let up = DVec3::Y;
        let base = Base {
            body,
            centre: up * radius,
            up,
            east: DVec3::X,
            north: DVec3::NEG_Z,
            wake: DVec3::new(-16.0, radius, 15.0),
            pad: DVec3::new(17.5, radius, -8.5).normalize() * radius,
            heading: DVec3::NEG_Z,
            door: DVec3::new(-16.0, radius, 6.0),
        };
        let height = |_: DVec3| 5.0;
        let (p0, _) = berth_bay(&base, &height, (radius, 0.0), 0, &[]);
        assert!((p0 - (base.pad + base.pad.normalize() * GEAR_DROP)).length() < 1e-4);
        let mut occupied = vec![p0];
        for slot in 1..6 {
            let (p, _) = berth_bay(&base, &height, (radius, 0.0), slot, &occupied);
            assert!(!base.covers(p, 5.9));
            assert!(occupied.iter().all(|o| o.distance(p) >= BAY_GAP - 1e-4));
            occupied.push(p);
        }
        let (rejoin_p0, _) = berth_bay(&base, &height, (radius, 0.0), 0, &[p0]);
        assert!(rejoin_p0.distance(p0) >= BAY_GAP - 1e-4);
    }
}
