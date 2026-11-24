//! Sweep-and-slide and overlap depenetration against nearby static world solids and ships.

use avian3d::prelude::*;
use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::*;

use super::Surround;
use super::math::{PUSH_PAD, depenetrate_offset, strip_inward};
use crate::fauna::beast::Beast;
use crate::fauna::bulk::capsule;
use crate::life::body::Body;
use crate::world::NOT_ANIMALS;

/// Sweeps a creature horizontally up to two slide rounds against nearby non-trimesh static solids.
pub(super) fn sweep(
    surround: &Surround,
    beast: &mut Beast,
    body: &Body,
    target_xz: Vec2,
    (player_entity, dt): (Option<Entity>, f32),
) -> Vec2 {
    let radius = capsule(body.length, body.height).0.max(0.2);
    let lift = body.height.mul_add(0.5, 0.0).max(radius).max(0.35);
    let step = (beast.motion.root.vel.at * dt).xz();
    let cur = beast.motion.root.at.xz() - step;
    let moved = slide(
        surround,
        beast,
        (cur, target_xz),
        (radius, lift),
        player_entity,
    );
    unstick(surround, beast, (moved, radius, lift), player_entity)
}

/// Runs up to two horizontal sphere-cast slide steps from `cur` toward `goal`.
fn slide(
    surround: &Surround,
    beast: &mut Beast,
    (mut cur, mut goal): (Vec2, Vec2),
    (radius, lift): (f32, f32),
    player_entity: Option<Entity>,
) -> Vec2 {
    let shape = Collider::sphere(radius);
    let filter = SpatialQueryFilter::from_mask(NOT_ANIMALS);
    let keeps = |e: Entity| is_solid(surround, e, player_entity);
    let to_render = surround.anchor.rotation_to_render(beast.spot.rot);
    let to_local = to_render.inverse();
    for _ in 0..2 {
        let delta = goal - cur;
        let Some(flat_dir) = delta.try_normalize() else {
            break;
        };
        let origin = probe_render(surround, beast, cur, lift);
        let Ok(way) = Dir3::new(to_render * Vec3::new(flat_dir.x, 0.0, flat_dir.y)) else {
            break;
        };
        let cfg = ShapeCastConfig {
            ignore_origin_penetration: true,
            ..ShapeCastConfig::from_max_distance(delta.length())
        };
        let Some(hit) = surround.query.cast_shape_predicate(
            &shape,
            origin,
            Quat::IDENTITY,
            way,
            &cfg,
            &filter,
            &keeps,
        ) else {
            return goal;
        };
        let n_xz = (to_local * hit.normal1).xz();
        if n_xz.length() < 0.25 {
            return goal;
        }
        let n = n_xz.normalize();
        let hit_at = cur + flat_dir * (hit.distance - PUSH_PAD).max(0.0);
        let rem = goal - hit_at;
        cur = hit_at;
        goal = hit_at + rem - n * rem.dot(n).min(0.0);
        strip_inward(&mut beast.motion.root.vel.at, n);
    }
    cur
}

/// Pushes a creature horizontally out of any overlapping non-trimesh solid collider.
fn unstick(
    surround: &Surround,
    beast: &mut Beast,
    (mut cur, radius, lift): (Vec2, f32, f32),
    player_entity: Option<Entity>,
) -> Vec2 {
    let shape = Collider::sphere(radius);
    let filter = SpatialQueryFilter::from_mask(NOT_ANIMALS);
    let to_render = surround.anchor.rotation_to_render(beast.spot.rot);
    let fallback = -(beast.motion.root.turn * Vec3::NEG_Z).xz();
    for _ in 0..2 {
        let probe_local = Vec3::new(cur.x, beast.motion.root.at.y + lift, cur.y);
        let probe = surround.anchor.to_render(beast.spot.planet(probe_local));
        let Some(entity) = first_overlap(surround, &shape, (probe, to_render), player_entity)
        else {
            break;
        };
        let only = |e: Entity| e == entity;
        let Some(proj) = surround
            .query
            .project_point_predicate(probe, false, &filter, &only)
        else {
            break;
        };
        let proj_local = beast.spot.local(surround.anchor.to_frame(proj.point));
        let push = depenetrate_offset(probe_local, proj_local, proj.is_inside, radius, fallback);
        if push.length_squared() <= 1e-8 {
            break;
        }
        cur += push;
        strip_inward(&mut beast.motion.root.vel.at, push);
    }
    cur
}

/// Returns the first non-trimesh solid collider overlapping `shape` at `(probe, rot)`.
fn first_overlap(
    surround: &Surround,
    shape: &Collider,
    (probe, rot): (Vec3, Quat),
    player_entity: Option<Entity>,
) -> Option<Entity> {
    let filter = SpatialQueryFilter::from_mask(NOT_ANIMALS);
    let mut hit = None;
    surround
        .query
        .shape_intersections_callback(shape, probe, rot, &filter, |e| {
            if is_solid(surround, e, player_entity) {
                hit = Some(e);
                false
            } else {
                true
            }
        });
    hit
}

/// Returns true when entity `e` is a solid non-terrain collider other than the local player.
fn is_solid(surround: &Surround, e: Entity, player_entity: Option<Entity>) -> bool {
    Some(e) != player_entity
        && surround
            .colliders
            .get(e)
            .is_ok_and(|c| c.shape().as_trimesh().is_none())
}

/// Returns the render-space centre of the horizontal collision sphere for `beast` at `(xz, lift)`.
fn probe_render(surround: &Surround, beast: &Beast, xz: Vec2, lift: f32) -> Vec3 {
    let local = Vec3::new(xz.x, beast.motion.root.at.y + lift, xz.y);
    surround.anchor.to_render(beast.spot.planet(local))
}
