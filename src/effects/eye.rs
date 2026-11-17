//! The eye: where the camera stands on the planet this frame.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use crate::render::GameCamera;
use crate::space::{Anchor, Frame, altitude, up};
use crate::world::{Globe, Terrain};

/// The camera this frame, read once so that every effect system agrees on it.
///
/// It comes from the transforms of this frame, not from the `GlobalTransform` of the last
/// one, which still holds the old render position on the frame of an anchor rebase.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Eye {
    /// Camera position in render space.
    pub at: Vec3,
    /// Camera orientation in render space.
    pub rot: Quat,
    /// Rotation from the local frame of the camera, with +Y up, to render axes.
    pub tilt: Quat,
    /// Height of the camera above sea level in meters.
    pub altitude: f32,
}

impl Default for Eye {
    /// Stands at the render origin of a flat world.
    fn default() -> Self {
        Self {
            at: Vec3::ZERO,
            rot: Quat::IDENTITY,
            tilt: Quat::IDENTITY,
            altitude: 0.0,
        }
    }
}

impl Eye {
    /// Returns the eye of a camera at render position `at` turned by `rot`, over `globe`.
    pub fn new(anchor: &Anchor, globe: &Globe, at: Vec3, rot: Quat) -> Self {
        let vertical = up(anchor, at).as_dvec3();
        Self {
            at,
            rot,
            tilt: DQuat::from_rotation_arc(DVec3::Y, vertical).as_quat(),
            altitude: altitude(anchor, globe, at),
        }
    }
}

/// Run condition: the anchor sits on a planet, so render positions can be read as ground.
pub fn grounded(anchor: Option<Res<Anchor>>, terrain: Option<Res<Terrain>>) -> bool {
    let on_ground = |anchor: Res<Anchor>| anchor.flat || matches!(anchor.frame, Frame::Surface(_));
    terrain.is_some() && anchor.is_some_and(on_ground)
}

/// Reads the camera of this frame into the eye.
pub fn watch(
    anchor: Res<Anchor>,
    terrain: Res<Terrain>,
    camera: Single<Entity, With<GameCamera>>,
    transforms: TransformHelper,
    mut eye: ResMut<Eye>,
) {
    if let Ok(pose) = transforms.compute_global_transform(*camera) {
        *eye = Eye::new(
            &anchor,
            terrain.globe(),
            pose.translation(),
            pose.rotation(),
        );
    }
}
