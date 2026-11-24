//! The flight around things: a flyer, a swarm member or a floater looks ahead along its way for the
//! colliders of the world (the base, the ship, rocks, trees, the near ground), rises and slows before
//! them, and never moves into one: a step that would end inside one stops at it.
//!
//! The rules are pure, over a `cast` that returns how far a ray from one point of the flat frame of an
//! animal toward another meets a collider; the system reads it from the colliders of the world.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::beast::{Beast, Part};
use super::bulk::capsule;
use super::motion::Motion;
use super::sync::RemoteFauna;
use super::tune::{LOOK, MAX_STEP, RISE};
use super::{Dead, Frozen, near, walk};
use crate::life::body::Locomotion;
use crate::screens::playing;
use crate::space::Anchor;
use crate::world::{NOT_ANIMALS, Terrain};

/// How far a ray from the first point toward the second meets a collider, in metres, within their gap.
pub type Cast<'a> = &'a dyn Fn(Vec3, Vec3) -> Option<f32>;

/// The parts of the animals, moved with a body stopped at a collider.
type Parts<'w, 's> = Query<'w, 's, &'static mut Transform, With<Part>>;

/// Returns true for the bodies that move in open air.
fn aloft(locomotion: Locomotion) -> bool {
    matches!(
        locomotion,
        Locomotion::Fly | Locomotion::Swarm | Locomotion::Pulse
    )
}

/// Takes back the last step of `m`, `dt` seconds at its velocity, as far as it went into a collider of
/// `cast` with a body `radius` metres round, stops its velocity into it, and returns the shift.
pub fn clamp(m: &mut Motion, cast: Cast, (radius, dt): (f32, f32)) -> Vec3 {
    let moved = m.root.vel.at * dt;
    let length = moved.length();
    let now = m.root.at;
    let old = now - moved;
    let hit = (length > 1e-4)
        .then(|| cast(old, now + moved / length.max(1e-4) * radius))
        .flatten();
    let Some(distance) = hit else {
        return Vec3::ZERO;
    };
    let keep = ((distance - radius) / length).clamp(0.0, 1.0);
    let back = moved * (keep - 1.0);
    let way = moved / length;
    let vel = m.root.vel.at;
    m.root.vel.at = vel - way * vel.dot(way).max(0.0);
    m.root.at += back;
    m.root.place += back;
    back
}

/// Looks [`LOOK`] seconds of its speed ahead of `m`, a body `radius` metres round, for a collider of
/// `cast`, and for `dt` seconds rises at up to [`RISE`] metres per second and slows, the more the nearer
/// it is; returns how far it rose.
pub fn look(m: &mut Motion, cast: Cast, (radius, dt): (f32, f32)) -> Vec3 {
    let vel = m.root.vel.at;
    let speed = vel.length();
    let reach = speed * LOOK + radius;
    let way = vel / speed.max(1e-3);
    let hit = (speed >= 0.05)
        .then(|| cast(m.root.at, m.root.at + way * reach))
        .flatten();
    let Some(distance) = hit else {
        return Vec3::ZERO;
    };
    let near = 1.0 - (distance / reach).clamp(0.0, 1.0);
    let lift = Vec3::Y * (RISE * near * dt);
    m.root.vel.at = vel - way * (0.5 * speed * near);
    m.root.at += lift;
    m.root.place += lift;
    lift
}

/// Steers every living drawn flyer, swarm member and floater clear of the colliders of the world after
/// the walk moved it, and moves its parts with it when a step was taken back.
pub fn clear(
    (terrain, anchor, time, remote): (
        Res<Terrain>,
        Res<Anchor>,
        Res<Time>,
        Option<Res<RemoteFauna>>,
    ),
    query: SpatialQuery,
    mut beasts: Query<&mut Beast, (Without<Frozen>, Without<Dead>)>,
    mut parts: Parts,
) {
    let Some(life) = terrain.globe().life() else {
        return;
    };
    let dt = time.delta_secs().min(MAX_STEP);
    let filter = SpatialQueryFilter::from_mask(NOT_ANIMALS);
    for mut beast in &mut beasts {
        if remote
            .as_deref()
            .is_some_and(|r| r.contains(beast.lead.key()))
        {
            continue;
        }
        let Some(body) = life.species.get(usize::from(beast.index)).map(|s| &s.body) else {
            continue;
        };
        if !aloft(body.locomotion) {
            continue;
        }
        let spot = beast.spot;
        let render = |local: Vec3| anchor.to_render(spot.planet(local));
        let cast = |from: Vec3, to: Vec3| {
            let (a, b) = (render(from), render(to));
            let way = Dir3::new(b - a).ok()?;
            query
                .cast_ray(a, way, a.distance(b), true, &filter)
                .map(|hit| hit.distance)
        };
        let (radius, _) = capsule(body.length, body.height);
        let back = clamp(&mut beast.motion, &cast, (radius, dt));
        let shift = back + look(&mut beast.motion, &cast, (radius, dt));
        for part in beast.parts.iter().flatten().filter(|_| shift != Vec3::ZERO) {
            if let Ok(mut place) = parts.get_mut(*part) {
                place.translation += shift;
            }
        }
    }
}

/// Registers the flight around things: after the walk, before the marks and the bodies follow it.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        clear
            .in_set(super::Posed)
            .after(walk::walk)
            .before(near::mark)
            .before(super::bulk::bulk_up)
            .run_if(playing)
            .run_if(super::gate::living)
            .run_if(any_with_component::<Beast>),
    );
}
