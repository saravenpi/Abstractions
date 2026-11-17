//! Spawning the player body, its camera, and the fall-out-of-world safety net.
//!
//! Spawn poses are chart positions of the terrain, whose `y` is the height above sea level:
//! adding to `y` lifts along the local up. They enter render space through the anchor.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::controller::Motion;
use super::upright::facing;
use super::{BODY_LENGTH, BODY_RADIUS, EYE_OFFSET, Player};
use crate::render::GameCamera;
use crate::saves::{ActiveWorld, WorldSave};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Rebased, altitude, on_foot, up};
use crate::world::{SpawnPoint, StreamAnchor, Terrain};

/// Height above the spawn point where the body appears.
const SPAWN_LIFT: f32 = 2.0;

/// Height above the spawn point where a resumed body appears: the point is its saved center.
const RESUME_LIFT: f32 = 0.05;

/// Altitude below which the player is teleported back to the spawn point.
const KILL_HEIGHT: f32 = -50.0;

/// Returns the chart position of the body for a spawn point.
fn body_position(spawn: SpawnPoint) -> Vec3 {
    spawn.0 + Vec3::Y * (SPAWN_LIFT + BODY_RADIUS + BODY_LENGTH * 0.5)
}

/// Chart position and view angles a new body starts with.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pose {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

/// Position in the planet frame the body returns to after falling out of the world.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(super) struct Home(pub(super) DVec3);

/// Returns the starting pose. A resumed world stands on the spawn point, which is then the saved
/// body center, with the saved angles. A new world drops in from above the spawn point.
fn start_pose(spawn: SpawnPoint, save: Option<&WorldSave>) -> Pose {
    match save.filter(|save| save.position.is_some() || save.place.is_some()) {
        Some(save) => Pose {
            position: spawn.0 + Vec3::Y * RESUME_LIFT,
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

/// Where the world puts the player: the spawn point and the way into render space.
#[derive(SystemParam)]
struct Site<'w> {
    spawn: Res<'w, SpawnPoint>,
    anchor: Res<'w, Anchor>,
    terrain: Res<'w, Terrain>,
}

impl Site<'_> {
    /// Returns the planet-frame position of a chart position.
    fn frame(&self, chart: Vec3) -> DVec3 {
        self.terrain.embed(chart.x, chart.y, chart.z)
    }
}

/// Returns the first-person camera, a child of the body that looks up and down by `pitch`.
fn eyes(settings: &Settings, pitch: f32) -> impl Bundle {
    (
        Name::new("PlayerCamera"),
        GameCamera,
        Rebased,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: (settings.graphics.fov as f32).to_radians(),
            near: 0.05,
            ..default()
        }),
        Transform::from_xyz(0.0, EYE_OFFSET, 0.0).with_rotation(Quat::from_rotation_x(pitch)),
    )
}

/// Spawns the player body with its first-person camera once the spawn point exists.
fn spawn_player(
    mut commands: Commands,
    site: Site,
    active: Option<Res<ActiveWorld>>,
    settings: Res<Persistent<Settings>>,
    existing: Query<(), With<Player>>,
) {
    if !existing.is_empty() {
        return;
    }
    let pose = start_pose(*site.spawn, active.as_ref().map(|active| &active.0.save));
    let home = site.frame(pose.position);
    let position = site.anchor.to_render(home);
    commands
        .spawn((
            Name::new("Player"),
            Player,
            StreamAnchor,
            Rebased,
            Home(home),
            Motion {
                yaw: pose.yaw,
                ..default()
            },
            DespawnOnEnter(Screen::Title),
            RigidBody::Kinematic,
            CustomPositionIntegration,
            Collider::capsule(BODY_RADIUS, BODY_LENGTH),
            Transform::from_translation(position)
                .with_rotation(facing(up(&site.anchor, position), pose.yaw)),
            Visibility::default(),
        ))
        .with_child(eyes(&settings, pose.pitch));
}

/// Teleports the player back to the spawn point after falling out of the world.
fn respawn_when_lost(
    anchor: Res<Anchor>,
    terrain: Res<Terrain>,
    mut player: Query<(&mut Transform, &mut Motion, &Home), With<Player>>,
) {
    for (mut transform, mut motion, home) in &mut player {
        if altitude(&anchor, terrain.globe(), transform.translation) < KILL_HEIGHT {
            transform.translation = anchor.to_render(home.0);
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
            respawn_when_lost.run_if(on_foot),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A resumed world stands on the spawn point with its saved angles; a new one drops in.
    #[test]
    fn resumed_pose_keeps_spot_and_angles() {
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
        assert_eq!(pose.position, Vec3::new(1.0, 2.0 + RESUME_LIFT, 3.0));
        assert_eq!((pose.yaw, pose.pitch), (1.0, -0.5));
        let unplayed = WorldSave::default();
        assert_eq!(start_pose(spawn, Some(&unplayed)), fresh);
    }
}
