//! The point chunks stream around: the anchor entity, or the spawn point before it exists.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{CHUNK_SIZE, SpawnPoint, StreamAnchor};

/// Reads the streaming focus.
#[derive(SystemParam)]
pub struct Focus<'w, 's> {
    anchor: Query<'w, 's, &'static Transform, With<StreamAnchor>>,
    spawn: Option<Res<'w, SpawnPoint>>,
}

impl Focus<'_, '_> {
    /// Returns the focus position in world space.
    pub fn position(&self) -> Option<Vec3> {
        self.anchor
            .iter()
            .next()
            .map(|t| t.translation)
            .or(self.spawn.as_ref().map(|s| s.0))
    }

    /// Returns the chunk containing the focus.
    pub fn chunk(&self) -> Option<IVec2> {
        self.position().map(|p| {
            IVec2::new(
                (p.x / CHUNK_SIZE).floor() as i32,
                (p.z / CHUNK_SIZE).floor() as i32,
            )
        })
    }
}
