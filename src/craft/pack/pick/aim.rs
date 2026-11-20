//! Aiming at a rock: a ray from the camera along the view, and the rock of a loaded chunk it hits.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::craft::pack::tune::{REACH, SIGHT};
use crate::player::Player;
use crate::render::GameCamera;
use crate::sim::PlayerId;
use crate::world::Chunks;
use crate::world::rocks::Rock;

/// What the aim reads: the camera, the local robot, the physics world and the loaded chunks.
#[derive(SystemParam)]
pub struct Aim<'w, 's> {
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    players: Query<'w, 's, (&'static GlobalTransform, &'static PlayerId), With<Player>>,
    space: SpatialQuery<'w, 's>,
    hidden: Query<'w, 's, (), With<Player>>,
    placed: Query<'w, 's, &'static GlobalTransform>,
    chunks: Res<'w, Chunks>,
}

impl Aim<'_, '_> {
    /// Returns the rock under the crosshair when it is within [`REACH`] of the local robot, and
    /// nothing when the ray meets the ground, a plant, or nothing first.
    pub fn rock(&self) -> Option<Rock> {
        let eye = self.camera.single().ok()?;
        let body = self
            .players
            .iter()
            .find(|(_, id)| **id == PlayerId::LOCAL)?;
        let dir = eye.forward();
        let filter = SpatialQueryFilter::default();
        let seen = |entity: Entity| !self.hidden.contains(entity);
        let hit =
            self.space
                .cast_ray_predicate(eye.translation(), dir, SIGHT, true, &filter, &seen)?;
        let point = eye.translation() + *dir * hit.distance;
        (point.distance(body.0.translation()) <= REACH).then_some(())?;
        let props = self.placed.get(hit.entity).ok()?;
        let local = props.affine().inverse().transform_point3(point);
        self.chunks.rock_under(hit.entity, local)
    }
}
