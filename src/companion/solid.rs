//! Touch: the companion is a small ball among the drawn solids, which stop it and let it slide.
//!
//! Each step of the follow is swept with avian's move and slide against every collider but
//! the player's body: a wall, a roof, a market stand and the ship's hull stop it at the hit
//! and it slides the rest of the step along the surface. Its spring keeps pulling it toward its
//! station along that slid path, so it never passes through what is drawn.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::motion::Float;
use super::tune::SHELL_RADIUS;
use crate::player::Player;

/// Share of its speed below which a step that hits a solid nearly head on has too little left
/// along the surface to slide anywhere, and turns toward the player instead.
const DETOUR: f32 = 0.3;

/// Returns the velocity `vel` of a step that hit a surface of outward normal `normal`, turned
/// along that surface toward `toward`, the way to the player, when it hit nearly head on.
///
/// Pulled straight at the middle of a wall, the slide alone leaves it stuck there; turned this
/// way it slides to the edge the player went round and follows on along the next face.
pub fn detour(vel: Vec3, normal: Vec3, toward: Vec3) -> Vec3 {
    let into = vel.dot(normal);
    let along = vel - normal * into;
    if into >= 0.0 || along.length() > DETOUR * vel.length() {
        return vel;
    }
    let side = (toward - normal * toward.dot(normal))
        .try_normalize()
        .unwrap_or_else(|| normal.any_orthonormal_vector());
    side * vel.length() + normal * into
}

/// The ball the companion is to the solids, built once.
pub struct Shell(Collider);

impl Default for Shell {
    /// Returns the ball of [`SHELL_RADIUS`].
    fn default() -> Self {
        Self(Collider::sphere(SHELL_RADIUS))
    }
}

/// What the sweep reads: avian's move and slide, the player bodies it passes beside, and the
/// ball, the filter and the settings it keeps from one frame to the next.
#[derive(SystemParam)]
pub struct Solids<'w, 's> {
    /// Avian's move and slide, valid in `Update`.
    slide: MoveAndSlide<'w, 's>,
    /// The player bodies, which the companion floats beside and never bumps into.
    players: Query<'w, 's, Entity, With<Player>>,
    /// The ball of the companion.
    shell: Local<'s, Shell>,
    /// The filter that leaves the player bodies out, rebuilt only when a new one appears.
    filter: Local<'s, SpatialQueryFilter>,
    /// The move and slide settings.
    config: Local<'s, MoveAndSlideConfig>,
}

impl Solids<'_, '_> {
    /// Moves the companion from `from` to where `at` stands now as a ball swept along that step
    /// of `dt` seconds: stopped at the first solid in the way and slid along it, round it toward
    /// the player's eyes at `eye` when it hit head on. The part of its velocity that pressed
    /// into the solid is taken out of `float`, and the turn round it is added.
    pub fn sweep(
        &mut self,
        (from, eye): (Vec3, Vec3),
        at: &mut Transform,
        float: &mut Float,
        dt: f32,
    ) {
        let known = &self.filter.excluded_entities;
        if self.players.iter().any(|body| !known.contains(&body)) {
            *self.filter = SpatialQueryFilter::from_excluded_entities(self.players.iter());
        }
        let dt = dt.max(1e-4);
        let velocity = (at.translation - from) / dt;
        let out = self.slide.move_and_slide(
            &self.shell.0,
            from,
            Quat::IDENTITY,
            velocity,
            Duration::from_secs_f32(dt),
            &self.config,
            &self.filter,
            |hit| {
                *hit.velocity = detour(*hit.velocity, hit.normal.as_vec3(), eye - *hit.position);
                MoveAndSlideHitResponse::Accept
            },
        );
        at.translation = out.position;
        float.vel += out.projected_velocity - velocity;
    }
}
