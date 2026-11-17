//! Kinematic first-person character controller built on avian's move and slide.
//!
//! The movement rules run in the upright frame of the body, where +Y is the local up.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::camera::mouse_look;
use super::dive::{SwimInput, swim_velocity, swim_wish};
use super::ground::{Ground, RISING_SPEED, probe, slope_velocity, snap_velocity, walkable};
use super::intent::{Intent, approach, land_vertical, read_intent};
use super::upright::{facing, pitch_of, tilt};
use super::{Player, swim};
use crate::render::GameCamera;
use crate::screens::playing;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, altitude, local_gravity, on_foot, up};
use crate::world::Terrain;

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
    /// Current velocity in the upright frame of the body: +Y is the local up.
    pub velocity: Vec3,
    /// Heading in radians around the local up. The body orientation follows it.
    pub yaw: f32,
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
    anchor: Res<'w, Anchor>,
    terrain: Res<'w, Terrain>,
    slide: MoveAndSlide<'w, 's>,
    query: SpatialQuery<'w, 's>,
}

/// Per-frame values shared by the movement steps.
struct Step {
    /// Simulated seconds.
    dt: f32,
    /// Seconds since startup, for the floating bob.
    time: f32,
    /// Body yaw around the local up.
    yaw: f32,
    /// Camera pitch.
    pitch: f32,
    /// Altitude of the body center above the sea.
    height: f32,
    /// Gravity at the body in meters per second squared.
    gravity: f32,
}

/// Returns the horizontal velocity for this step.
fn horizontal(motion: &Motion, intent: &Intent, step: &Step) -> Vec3 {
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
        intent.wish * base * swim::wade_factor(step.height),
        rate,
        step.dt,
    );
    if intent.wish == Vec3::ZERO && next.length() < REST_SPEED {
        Vec3::ZERO
    } else {
        next
    }
}

/// Returns the walking velocity for this step, glued to walkable ground.
fn walk_step(motion: &mut Motion, intent: &Intent, ground: Option<Ground>, step: &Step) -> Vec3 {
    motion.grounded = ground.is_some();
    let flat = horizontal(motion, intent, step);
    let vertical = land_vertical(motion, intent, step.gravity, step.dt);
    match ground.filter(|_| motion.grounded) {
        Some(ground) => {
            slope_velocity(flat, ground.normal) + Vec3::Y * snap_velocity(ground.gap, step.dt)
        }
        None => flat + Vec3::Y * vertical,
    }
}

/// Chooses the swimming or walking velocity for this frame and updates the swim state.
fn velocity_for(motion: &mut Motion, intent: &Intent, floor: Option<Ground>, step: &Step) -> Vec3 {
    let gap = walkable(floor, false).map(|ground| ground.gap);
    motion.swimming = swim::next_swimming(motion.swimming, step.height, gap);
    if !motion.swimming {
        let ground = walkable(floor, motion.velocity.y > RISING_SPEED);
        return walk_step(motion, intent, ground, step);
    }
    motion.grounded = false;
    let input = SwimInput {
        wish: swim_wish(intent.forward, intent.right, step.yaw, step.pitch),
        sprint: intent.sprint,
        up: intent.jump,
    };
    swim_velocity(motion.velocity, &input, step.height, step.time, step.dt)
}

/// Reads the per-frame values of a body at render position `at` heading `yaw`, seen through `camera`.
fn read_step(env: &Env, at: Vec3, yaw: f32, camera: &Transform) -> Step {
    let globe = env.terrain.globe();
    Step {
        dt: env.time.delta_secs().min(MAX_DT),
        time: env.time.elapsed_secs(),
        yaw,
        pitch: pitch_of(camera),
        height: altitude(&env.anchor, globe, at),
        gravity: local_gravity(&env.anchor, globe, at).length(),
    }
}

/// Runs one step of input, ground detection and collision-aware movement.
fn drive(
    env: Env,
    player: Single<(Entity, &mut Transform, &mut Motion, &Collider), With<Player>>,
    camera: Single<&Transform, (With<GameCamera>, Without<Player>)>,
) {
    let (entity, mut transform, mut motion, collider) = player.into_inner();
    let step = read_step(&env, transform.translation, motion.yaw, &camera);
    let to_render = tilt(up(&env.anchor, transform.translation));
    let intent = read_intent(&env.keys, &env.settings.controls.keys, step.yaw);
    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let floor = probe(&env.query, collider, &transform, to_render, &filter);
    let velocity = velocity_for(&mut motion, &intent, floor, &step);
    let out = env.slide.move_and_slide(
        collider,
        transform.translation,
        transform.rotation,
        to_render * velocity,
        Duration::from_secs_f32(step.dt),
        &MoveAndSlideConfig::default(),
        &filter,
        |_| MoveAndSlideHitResponse::Accept,
    );
    transform.translation = out.position;
    transform.rotation = facing(up(&env.anchor, out.position), step.yaw);
    let moved = to_render.inverse() * out.projected_velocity;
    motion.velocity = if motion.grounded {
        moved.with_y(0.0)
    } else {
        moved
    };
}

/// Registers the controller system.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        drive.after(mouse_look).run_if(playing).run_if(on_foot),
    );
}
