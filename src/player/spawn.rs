//! Spawning the player body, its camera, and the fall-out-of-world safety net.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::controller::Motion;
use super::{BODY_LENGTH, BODY_RADIUS, EYE_OFFSET, Player};
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::world::{SpawnPoint, StreamAnchor};

/// Height above the spawn point where the body appears.
const SPAWN_LIFT: f32 = 2.0;

/// Depth below which the player is teleported back to the spawn point.
const KILL_HEIGHT: f32 = -50.0;

/// Vertical field of view of the player camera, in degrees.
const FOV_DEGREES: f32 = 75.0;

/// Returns the body position for a spawn point.
fn body_position(spawn: SpawnPoint) -> Vec3 {
    spawn.0 + Vec3::Y * (SPAWN_LIFT + BODY_RADIUS + BODY_LENGTH * 0.5)
}

/// Spawns the player body with its first-person camera once the spawn point exists.
fn spawn_player(mut commands: Commands, spawn: Res<SpawnPoint>, existing: Query<(), With<Player>>) {
    if !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            Name::new("Player"),
            Player,
            StreamAnchor,
            Motion::default(),
            DespawnOnEnter(Screen::Title),
            RigidBody::Kinematic,
            CustomPositionIntegration,
            Collider::capsule(BODY_RADIUS, BODY_LENGTH),
            Transform::from_translation(body_position(*spawn)),
            Visibility::default(),
        ))
        .with_child((
            Name::new("PlayerCamera"),
            GameCamera,
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: FOV_DEGREES.to_radians(),
                near: 0.05,
                ..default()
            }),
            Transform::from_xyz(0.0, EYE_OFFSET, 0.0),
        ));
}

/// Teleports the player back to the spawn point after falling out of the world.
fn respawn_when_lost(
    spawn: Res<SpawnPoint>,
    mut player: Query<(&mut Transform, &mut Motion), With<Player>>,
) {
    for (mut transform, mut motion) in &mut player {
        if transform.translation.y < KILL_HEIGHT {
            transform.translation = body_position(*spawn);
            motion.velocity = Vec3::ZERO;
        }
    }
}

/// Registers spawning and respawn systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_player.run_if(resource_added::<SpawnPoint>),
            respawn_when_lost.run_if(resource_exists::<SpawnPoint>),
        ),
    );
}
