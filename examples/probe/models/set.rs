//! The studio scene and the poses: ground, white light, the robot, the companion and the camera.

use abstractions::avatar::strike::Layer;
use abstractions::avatar::{AvatarPose, Looks, Mode, puppet};
use abstractions::companion::{Face, Float, Pose as Hover};
use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use super::plan::{Mood, Shot, Stance, View, Who};
use super::*;

/// Height of the robot's body centre over the ground, meters.
const STAND: f32 = 0.9;

/// Height of a hovering companion over the ground, meters.
const HOVER: f32 = 1.25;

/// The robot of the studio.
#[derive(Component)]
pub struct Robot;

/// The companion of the studio.
#[derive(Component)]
pub struct Buddy;

/// Spawns the ground, the light and the camera, and builds the shared look of the robot.
pub fn stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground = materials.add(Color::srgb(0.55, 0.56, 0.54));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(60.0, 60.0))),
        MeshMaterial3d(ground),
    ));
    let sun = DirectionalLight {
        illuminance: 10_000.0,
        shadow_maps_enabled: true,
        ..default()
    };
    let aim = Transform::from_xyz(-3.0, 8.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((sun, aim));
    commands.insert_resource(GlobalAmbientLight {
        brightness: 500.0,
        ..default()
    });
    commands.insert_resource(Looks::new(&mut meshes, &mut materials));
    let lens = Projection::Perspective(PerspectiveProjection {
        fov: 60f32.to_radians(),
        ..default()
    });
    commands.spawn((Camera3d::default(), lens, Transform::default()));
}

/// Spawns the robot from the shared look.
pub fn robot(mut commands: Commands, mut stock: abstractions::avatar::Stock, looks: Res<Looks>) {
    let body = puppet(&mut commands, &mut stock, &looks, None);
    commands.entity(body).insert(Robot);
}

/// Spawns the companion.
pub fn buddy(mut commands: Commands, mut stock: abstractions::companion::Stock) {
    let bot = abstractions::companion::look(&mut commands, &mut stock, Transform::IDENTITY);
    commands.entity(bot).insert((Buddy, Face::new(5)));
}

/// Returns the pose of the robot for `stance` at frame `frame`.
fn robot_pose(stance: Stance, frame: f32) -> AvatarPose {
    let base = AvatarPose::default();
    let walking = |mode, speed: f32| AvatarPose {
        mode,
        speed,
        stride_distance: frame * speed * super::STEP,
        ..base
    };
    match stance {
        Stance::Idle | Stance::Tool | Stance::Blade | Stance::Spear => base,
        Stance::Walk => walking(Mode::Walk, 1.6),
        Stance::Run => walking(Mode::Run, 6.5),
        Stance::Crouch => AvatarPose { crouch: 1.0, ..base },
        Stance::Glide => AvatarPose {
            mode: Mode::Glide,
            wings: 1.0,
            prone: 1.0,
            speed: 28.0,
            airspeed: 28.0,
            clearance: 50.0,
            ..base
        },
        Stance::Seat => AvatarPose { mode: Mode::Seated, ..base },
        Stance::Swim => AvatarPose { mode: Mode::Swim, speed: 1.0, ..base },
    }
}

/// Returns the arms that hold what `stance` holds, if it holds anything: shoulder, lift and elbow.
fn grip(stance: Stance) -> Option<Layer> {
    let arms = match stance {
        Stance::Tool => [None, Some([0.9, 0.15, 1.0])],
        Stance::Blade => [Some([1.0, -0.1, 1.2]), Some([1.1, 0.2, 1.1])],
        Stance::Spear => [None, Some([1.4, 0.25, 0.7])],
        _ => return None,
    };
    Some(Layer {
        weight: 1.0,
        trunk: [0.0, 0.0],
        arms,
    })
}

/// Returns the pose of the companion for `mood`.
fn buddy_pose(mood: Mood) -> (Float, Option<abstractions::companion::Expression>) {
    let calm = Float::default();
    match mood {
        Mood::Hover => (calm, None),
        Mood::Follow => (
            Float {
                vel: Vec3::NEG_Z * 6.0,
                flight: 1.0,
                ..calm
            },
            None,
        ),
        Mood::Ride => (Float { seated: true, ..calm }, None),
        Mood::Scan => (
            Float {
                pose: Hover {
                    spread: 1.0,
                    pitch: 0.2,
                    ..default()
                },
                ..calm
            },
            None,
        ),
        Mood::Show(e) => (calm, Some(e)),
    }
}

/// Returns where the camera stands and what it looks at for `shot`.
fn camera(shot: &Shot, aim: Vec3) -> Transform {
    let eye = match shot.view {
        View::Around(deg) => {
            Quat::from_rotation_y(deg.to_radians()) * Vec3::new(0.0, 0.15, -1.0).normalize()
        }
        View::Top => Vec3::Y,
    };
    let up = if shot.view == View::Top { Vec3::NEG_Z } else { Vec3::Y };
    Transform::from_translation(aim + eye * shot.dist).looking_at(aim, up)
}

/// Sets the scene up for the shot in hand and keeps its walking going.
pub fn advance(
    mut commands: Commands,
    plan: Res<Plan>,
    mut robot: Query<(Entity, &mut AvatarPose, &mut Transform), With<Robot>>,
    mut buddy: Query<(&mut Float, &mut Face, &mut Transform), (With<Buddy>, Without<Robot>)>,
    mut eye: Query<&mut Transform, (With<Camera3d>, Without<Robot>, Without<Buddy>)>,
) {
    let Some(shot) = plan.todo.last() else {
        return;
    };
    let (Ok((body, mut pose, mut at)), Ok((mut float, mut face, mut bat))) =
        (robot.single_mut(), buddy.single_mut())
    else {
        return;
    };
    let aloft = matches!(shot.stance, Stance::Glide);
    let alone = shot.who == Who::Buddy;
    let lift = if aloft { 4.0 } else { 0.0 };
    at.translation = Vec3::new(if alone { 20.0 } else { 0.0 }, STAND + lift, 0.0);
    *pose = robot_pose(shot.stance, plan.waited as f32);
    match grip(shot.stance) {
        Some(layer) => commands.entity(body).insert(layer),
        None => commands.entity(body).remove::<Layer>(),
    };
    let (moved, show) = buddy_pose(shot.mood);
    *float = moved;
    if let Some(e) = show {
        face.react(e, 100.0);
        face.advance(abstractions::companion::Expression::Neutral, super::STEP);
    }
    let beside = Vec3::new(0.75, HOVER, -0.5);
    bat.translation = match shot.who {
        Who::Buddy => Vec3::new(0.0, HOVER, 0.0),
        _ => at.translation + beside - Vec3::Y * (STAND - 0.1),
    };
    let aim = match shot.who {
        Who::Robot => at.translation,
        Who::Buddy => bat.translation,
        Who::Both => at.translation + Vec3::new(0.35, 0.1, -0.2),
    };
    if let Ok(mut cam) = eye.single_mut() {
        *cam = camera(shot, aim);
    }
}
