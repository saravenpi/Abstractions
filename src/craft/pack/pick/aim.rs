//! Aiming at a rock, tree, or diggable ground under the crosshair within interact reach.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::craft::pack::tune::{REACH, SIGHT};
use crate::player::Player;
use crate::render::GameCamera;
use crate::sim::PlayerId;
use crate::world::Chunks;
use crate::world::rocks::Rock;
use crate::world::trees::Tree;

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
    /// Casts a reach-bounded ray along the camera forward axis, returning hit entity and world point.
    fn ray(&self) -> Option<(Entity, Vec3)> {
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
        (point.distance(body.0.translation()) <= REACH).then_some((hit.entity, point))
    }

    /// Returns the rock under the crosshair when within [`REACH`] of the local robot.
    pub fn rock(&self) -> Option<(Rock, Vec3)> {
        let (entity, point) = self.ray()?;
        let props = self.placed.get(entity).ok()?;
        let local = props.affine().inverse().transform_point3(point);
        self.chunks
            .rock_under(entity, local)
            .map(|rock| (rock, point))
    }

    /// Returns the standing tree under the crosshair when within [`REACH`] of the local robot.
    pub fn tree(&self) -> Option<(Tree, Vec3)> {
        let (entity, point) = self.ray()?;
        let props = self.placed.get(entity).ok()?;
        let local = props.affine().inverse().transform_point3(point);
        self.chunks
            .tree_under(entity, local)
            .map(|tree| (tree, point))
    }

    /// Returns the world point on loaded terrain under the crosshair when within [`REACH`].
    pub fn ground(&self) -> Option<Vec3> {
        let (entity, point) = self.ray()?;
        self.chunks.is_terrain(entity).then_some(point)
    }
}
