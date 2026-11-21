//! The sight of the local robot: the camera ray through the crosshair, the part of an animal it
//! meets first, and the point a strike aims at. A strike goes where the player points, down to a
//! small animal at the feet, within the reach of the tool; the crosshair reads the same sight.

use bevy::prelude::*;

use super::super::super::tune::{FOREARM, SOLE_Y, UPPER_ARM};
use super::super::act::{Act, Strike};
use super::super::held::Held;
use super::super::hit::{Bounds, enter};
use super::super::input::Robot;
use super::super::sense::Parts;
use super::bend::shoulder;
use super::tune::{AIM_PAD, AIM_SIGHT, GROUND_CLEAR, LUNGE};
use crate::fauna::Live;
use crate::render::GameCamera;

/// What the crosshair of the local robot points at.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Sight {
    /// The animal and its part under the crosshair, if any.
    pub target: Option<(Entity, Entity)>,
    /// True when that animal is within the reach of the held tool.
    pub reach: bool,
    /// Where a strike aims, in render space, while a tool is held.
    pub point: Option<Vec3>,
}

/// Returns where the ray from `eye` along the unit `dir` leaves the sphere of `radius` about
/// `centre`, or the point of the sphere nearest the ray when the ray misses it ahead.
pub fn exit(eye: Vec3, dir: Vec3, (centre, radius): (Vec3, f32)) -> Vec3 {
    let to = eye - centre;
    let b = to.dot(dir);
    let disc = b * b - (to.length_squared() - radius * radius);
    let far = -b + disc.max(0.0).sqrt();
    if disc >= 0.0 && far > 0.0 {
        return eye + dir * far;
    }
    let near = eye + dir * (-b).max(0.0);
    centre + (near - centre).normalize_or(dir) * radius
}

/// Returns the part of an animal among `parts` that the ray from `eye` along the unit `dir` meets
/// first within `far` metres, with its animal and the metres along the ray. Each part is padded by
/// `pad` metres, so a small animal near the crosshair is under it.
pub fn on_ray(
    (parts, creatures): (&Parts, &Query<(), Live>),
    (eye, dir, far): (Vec3, Vec3, f32),
    pad: f32,
) -> Option<(Entity, Entity, f32)> {
    let end = eye + dir * far;
    let mut best: Option<(Entity, Entity, f32)> = None;
    for (part, placed, aabb, of) in parts.iter() {
        let near = placed.translation().distance(eye) < far + 2.0;
        if !near || creatures.get(of.parent()).is_err() {
            continue;
        }
        let inverse = placed.affine().inverse();
        let scale = placed.scale().abs().max(Vec3::splat(1e-3));
        let bounds = Bounds {
            centre: aabb.center.into(),
            half: aabb.half_extents.into(),
        };
        let (from, to) = (inverse.transform_point3(eye), inverse.transform_point3(end));
        let Some(s) = enter(from, to, &bounds, Vec3::splat(pad) / scale) else {
            continue;
        };
        if best.is_none_or(|(_, _, metres)| s * far < metres) {
            best = Some((part, of.parent(), s * far));
        }
    }
    best
}

/// Returns true while `act` may still change where a strike aims: before the blow is under way.
fn aiming(act: Act) -> bool {
    matches!(
        act,
        Act::Ready | Act::Windup(_) | Act::Guard(_) | Act::Draw(_)
    )
}

/// Reads where the crosshair of the local robot points: the animal under it, whether the held tool
/// reaches it, and the point a strike aims at, which the strike of the robot takes until its blow
/// is under way. With no animal in reach the aim runs along the view to the edge of the reach, so
/// a strike follows the pitch of the camera; it never aims into the ground.
pub fn look(
    (robot, camera): (Robot, Query<&GlobalTransform, With<GameCamera>>),
    (parts, creatures): (Parts, Query<(), Live>),
    mut bodies: Query<(&GlobalTransform, &mut Strike, &Held)>,
    mut sight: ResMut<Sight>,
) {
    let found = robot.body().and_then(|body| bodies.get_mut(body).ok());
    let (Some((placed, mut strike, held)), Ok(eye)) = (found, camera.single()) else {
        sight.set_if_neq(Sight::default());
        return;
    };
    let reach = held.build.reach() + UPPER_ARM + FOREARM + LUNGE;
    let pivot = placed.transform_point(shoulder());
    let (origin, dir) = (eye.translation(), eye.forward().as_vec3());
    let target = on_ray((&parts, &creatures), (origin, dir, AIM_SIGHT), AIM_PAD);
    let near = target
        .map(|(_, _, metres)| origin + dir * metres)
        .filter(|point| point.distance(pivot) <= reach);
    let point = near.unwrap_or_else(|| exit(origin, dir, (pivot, reach)));
    sight.set_if_neq(Sight {
        target: target.map(|(part, animal, _)| (animal, part)),
        reach: near.is_some(),
        point: Some(point),
    });
    if held.out && aiming(strike.act) {
        let local = placed.affine().inverse().transform_point3(point);
        strike.aim = Some(local.with_y(local.y.max(SOLE_Y + GROUND_CLEAR)));
    }
}
