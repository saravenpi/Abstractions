//! Kinematic first-person character controller built on avian's move and slide.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::dive::{SwimInput, swim_velocity, swim_wish};
use super::ground::{Ground, RISING_SPEED, probe, slope_velocity, snap_velocity, walkable};
use super::intent::{Intent, approach, land_vertical, read_intent};
use super::{Player, swim};
use crate::render::GameCamera;
use crate::screens::playing;
use crate::settings::{Persistent, Settings};

/// Walking speed in meters per second.
const WALK_SPEED: f32 = 5.0;

/// Sprinting speed in meters per second.
const SPRINT_SPEED: f32 = 8.5;

/// Horizontal acceleration rate on the ground.
const GROUND_ACCEL: f32 = 14.0;

/// Horizontal acceleration rate in the air.
const AIR_ACCEL: f32 = 3.5;

/// Longest simulated frame, to survive hitches.
const MAX_DT: f32 = 0.05;

/// Horizontal speed below which an idle body stops dead.
const REST_SPEED: f32 = 0.05;

/// Movement state owned by the controller.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Motion {
    /// Current velocity in world space.
    pub velocity: Vec3,
    /// Whether walkable ground was touched on the last step.
    pub grounded: bool,
    /// Remaining seconds in which a jump is still allowed.
    pub coyote: f32,
    /// Whether the body is swimming instead of walking.
    pub swimming: bool,
}

/// Services the controller needs besides the player query.
#[derive(SystemParam)]
struct Env<'w, 's> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    settings: Res<'w, Persistent<Settings>>,
    slide: MoveAndSlide<'w, 's>,
    query: SpatialQuery<'w, 's>,
}

/// Returns the horizontal velocity for this step.
fn horizontal(motion: &Motion, intent: &Intent, center_y: f32, dt: f32) -> Vec3 {
    let base = if intent.sprint {
        SPRINT_SPEED
    } else {
        WALK_SPEED
    };
    let rate = if motion.grounded {
        GROUND_ACCEL
    } else {
        AIR_ACCEL
    };
    let current = Vec3::new(motion.velocity.x, 0.0, motion.velocity.z);
    let next = approach(
        current,
        intent.wish * base * swim::wade_factor(center_y),
        rate,
        dt,
    );
    if intent.wish == Vec3::ZERO && next.length() < REST_SPEED {
        Vec3::ZERO
    } else {
        next
    }
}

/// Returns the walking velocity for this step, glued to walkable ground.
fn walk_step(
    motion: &mut Motion,
    intent: &Intent,
    ground: Option<Ground>,
    center_y: f32,
    dt: f32,
) -> Vec3 {
    motion.grounded = ground.is_some();
    let flat = horizontal(motion, intent, center_y, dt);
    let vertical = land_vertical(motion, intent, dt);
    match ground.filter(|_| motion.grounded) {
        Some(ground) => {
            slope_velocity(flat, ground.normal) + Vec3::Y * snap_velocity(ground.gap, dt)
        }
        None => flat + Vec3::Y * vertical,
    }
}

/// Per-frame values shared by the movement steps.
struct Frame {
    dt: f32,
    time: f32,
    yaw: f32,
    pitch: f32,
    center_y: f32,
}

/// Chooses the swimming or walking velocity for this frame and updates the swim state.
fn velocity_for(
    motion: &mut Motion,
    intent: &Intent,
    floor: Option<Ground>,
    frame: &Frame,
) -> Vec3 {
    let gap = walkable(floor, false).map(|ground| ground.gap);
    motion.swimming = swim::next_swimming(motion.swimming, frame.center_y, gap);
    if !motion.swimming {
        let ground = walkable(floor, motion.velocity.y > RISING_SPEED);
        return walk_step(motion, intent, ground, frame.center_y, frame.dt);
    }
    motion.grounded = false;
    let input = SwimInput {
        wish: swim_wish(intent.forward, intent.right, frame.yaw, frame.pitch),
        sprint: intent.sprint,
        up: intent.jump,
    };
    swim_velocity(
        motion.velocity,
        &input,
        frame.center_y,
        frame.time,
        frame.dt,
    )
}

/// Returns the body yaw and the camera pitch.
fn view_angles(body: &Transform, camera: &Transform) -> (f32, f32) {
    let (yaw, _, _) = body.rotation.to_euler(EulerRot::YXZ);
    let (_, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    (yaw, pitch)
}

/// Runs one step of input, ground detection and collision-aware movement.
fn drive(
    env: Env,
    player: Single<(Entity, &mut Transform, &mut Motion, &Collider), With<Player>>,
    camera: Single<&Transform, (With<GameCamera>, Without<Player>)>,
) {
    let (entity, mut transform, mut motion, collider) = player.into_inner();
    let (yaw, pitch) = view_angles(&transform, &camera);
    let frame = Frame {
        dt: env.time.delta_secs().min(MAX_DT),
        time: env.time.elapsed_secs(),
        yaw,
        pitch,
        center_y: transform.translation.y,
    };
    let intent = read_intent(&env.keys, &env.settings.controls.keys, yaw);
    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let floor = probe(&env.query, collider, &transform, &filter);
    let velocity = velocity_for(&mut motion, &intent, floor, &frame);
    let out = env.slide.move_and_slide(
        collider,
        transform.translation,
        transform.rotation,
        velocity,
        Duration::from_secs_f32(frame.dt),
        &MoveAndSlideConfig::default(),
        &filter,
        |_| MoveAndSlideHitResponse::Accept,
    );
    transform.translation = out.position;
    let moved = out.projected_velocity;
    motion.velocity = if motion.grounded {
        moved.with_y(0.0)
    } else {
        moved
    };
}

/// Registers the controller system.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, drive.run_if(playing));
}
