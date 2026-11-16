//! Spawning the player body, its camera, and the fall-out-of-world safety net.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::controller::Motion;
use super::{BODY_LENGTH, BODY_RADIUS, EYE_OFFSET, Player};
use crate::render::GameCamera;
use crate::saves::{ActiveWorld, WorldSave};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::world::{SpawnPoint, StreamAnchor};

/// Height above the spawn point where the body appears.
const SPAWN_LIFT: f32 = 2.0;

/// Height above a saved body position where a resumed body appears.
const RESUME_LIFT: f32 = 0.05;

/// Depth below which the player is teleported back to the spawn point.
const KILL_HEIGHT: f32 = -50.0;

/// Returns the body position for a spawn point.
fn body_position(spawn: SpawnPoint) -> Vec3 {
    spawn.0 + Vec3::Y * (SPAWN_LIFT + BODY_RADIUS + BODY_LENGTH * 0.5)
}

/// Position and view angles a new body starts with.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pose {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

/// Returns the starting pose: the saved one for a resumed world, else above the spawn point.
fn start_pose(spawn: SpawnPoint, save: Option<&WorldSave>) -> Pose {
    match save.and_then(|save| save.position.map(|at| (at, save))) {
        Some((at, save)) => Pose {
            position: Vec3::from(at) + Vec3::Y * RESUME_LIFT,
            yaw: save.yaw,
            pitch: save.pitch,
        },
        None => Pose {
            position: body_position(spawn),
            yaw: 0.0,
            pitch: 0.0,
        },
    }
}

/// Spawns the player body with its first-person camera once the spawn point exists.
fn spawn_player(
    mut commands: Commands,
    spawn: Res<SpawnPoint>,
    active: Option<Res<ActiveWorld>>,
    settings: Res<Persistent<Settings>>,
    existing: Query<(), With<Player>>,
) {
    if !existing.is_empty() {
        return;
    }
    let pose = start_pose(*spawn, active.as_ref().map(|active| &active.0.save));
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
            Transform::from_translation(pose.position)
                .with_rotation(Quat::from_rotation_y(pose.yaw)),
            Visibility::default(),
        ))
        .with_child((
            Name::new("PlayerCamera"),
            GameCamera,
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: (settings.graphics.fov as f32).to_radians(),
                near: 0.05,
                ..default()
            }),
            Transform::from_xyz(0.0, EYE_OFFSET, 0.0)
                .with_rotation(Quat::from_rotation_x(pose.pitch)),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A saved position wins over the spawn point and carries its angles.
    #[test]
    fn saved_pose_overrides_spawn() {
        let spawn = SpawnPoint(Vec3::new(1.0, 2.0, 3.0));
        let fresh = start_pose(spawn, None);
        assert_eq!(fresh.position, body_position(spawn));
        let save = WorldSave {
            position: Some([5.0, 6.0, 7.0]),
            yaw: 1.0,
            pitch: -0.5,
            ..WorldSave::default()
        };
        let pose = start_pose(spawn, Some(&save));
        assert_eq!(pose.position, Vec3::new(5.0, 6.0 + RESUME_LIFT, 7.0));
        assert_eq!((pose.yaw, pose.pitch), (1.0, -0.5));
        let unplayed = WorldSave::default();
        assert_eq!(start_pose(spawn, Some(&unplayed)), fresh);
    }
}
