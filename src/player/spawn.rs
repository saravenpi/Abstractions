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
use crate::gear::Gear;
use crate::render::GameCamera;
use crate::saves::{ActiveWorld, WorldSave};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Rebased, altitude, on_foot, up};
use crate::view::{Cut, Heading};
use crate::world::{SpawnPoint, StreamAnchor, Terrain};

/// Height above the spawn point where the body appears.
const SPAWN_LIFT: f32 = 2.0;

/// Height above the spawn point where a resumed body appears: the point is its saved center.
pub(super) const RESUME_LIFT: f32 = 0.05;

/// Altitude below which the player is teleported back to the spawn point.
const KILL_HEIGHT: f32 = -50.0;

/// Returns the chart position of the body for a spawn point.
pub(super) fn body_position(spawn: SpawnPoint) -> Vec3 {
    spawn.0 + Vec3::Y * (SPAWN_LIFT + BODY_RADIUS + BODY_LENGTH * 0.5)
}

/// Chart position and view angles a new body starts with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Pose {
    /// Chart position of the body centre.
    pub(super) position: Vec3,
    /// Heading around the local up.
    pub(super) yaw: f32,
    /// Pitch of the view.
    pub(super) pitch: f32,
}

/// Position in the planet frame the body returns to after falling out of the world.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(super) struct Home(pub(super) DVec3);

/// Returns the starting pose. A resumed world stands on the spawn point, which is then the saved
/// body center, with the saved angles. A new world drops in from above the spawn point.
pub(super) fn start_pose(spawn: SpawnPoint, save: Option<&WorldSave>) -> Pose {
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

/// Returns the game camera, a child of the body. It starts at the eyes, looking up or down by
/// `pitch`; from the first frame of play the camera rig places it.
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

/// Returns the player body, standing at `at` with its home at the planet-frame `home` and its
/// heading from `pose`.
fn body(home: DVec3, pose: &Pose, at: Transform) -> impl Bundle {
    (
        (
            Name::new("Player"),
            Player,
            StreamAnchor,
            Rebased,
            Home(home),
        ),
        Motion {
            yaw: pose.yaw,
            ..default()
        },
        Gear::default(),
        DespawnOnEnter(Screen::Title),
        RigidBody::Kinematic,
        CustomPositionIntegration,
        Collider::capsule(BODY_RADIUS, BODY_LENGTH),
        at,
        Visibility::default(),
    )
}

/// Spawns the player body with its camera once the spawn point exists, and has the camera rig
/// start on the angles of the pose: a new body is a cut, not a move.
fn spawn_player(
    mut commands: Commands,
    site: Site,
    (active, settings): (Option<Res<ActiveWorld>>, Res<Persistent<Settings>>),
    existing: Query<(), With<Player>>,
    mut cuts: MessageWriter<Cut>,
) {
    if !existing.is_empty() {
        return;
    }
    let pose = start_pose(*site.spawn, active.as_ref().map(|active| &active.0.save));
    let home = site.frame(pose.position);
    let position = site.anchor.to_render(home);
    let heading = Heading {
        yaw: pose.yaw,
        pitch: pose.pitch,
    };
    cuts.write(Cut {
        heading: Some(heading),
    });
    let at = Transform::from_translation(position)
        .with_rotation(facing(up(&site.anchor, position), pose.yaw));
    commands
        .spawn(body(home, &pose, at))
        .with_child(eyes(&settings, pose.pitch));
}

/// Teleports the player back to the spawn point after falling out of the world: at rest, the
/// flight gear folded, and with a cut of the camera.
fn respawn_when_lost(
    anchor: Res<Anchor>,
    terrain: Res<Terrain>,
    mut player: Query<(&mut Transform, &mut Motion, &mut Gear, &Home), With<Player>>,
    mut cuts: MessageWriter<Cut>,
) {
    for (mut transform, mut motion, mut gear, home) in &mut player {
        if altitude(&anchor, terrain.globe(), transform.translation) < KILL_HEIGHT {
            transform.translation = anchor.to_render(home.0);
            *motion = Motion {
                yaw: motion.yaw,
                ..default()
            };
            *gear = Gear::default();
            cuts.write(Cut::default());
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
