//! Kinematic first-person character controller built on avian's move and slide.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::intent::{Intent, approach, land_vertical, read_intent};
use super::{Player, swim};
use crate::screens::playing;
use crate::settings::{Persistent, Settings};

/// Walking speed in meters per second.
const WALK_SPEED: f32 = 5.0;

/// Sprinting speed in meters per second.
const SPRINT_SPEED: f32 = 8.5;

/// Smallest surface normal height that counts as walkable ground (about 50 degrees).
const MIN_GROUND_NORMAL: f32 = 0.64;

/// Speed multiplier in water.
const SWIM_FACTOR: f32 = 0.5;

/// Horizontal acceleration rate on the ground.
const GROUND_ACCEL: f32 = 14.0;

/// Horizontal acceleration rate in the air and in water.
const AIR_ACCEL: f32 = 3.5;

/// Longest simulated frame, to survive hitches.
const MAX_DT: f32 = 0.05;

/// Movement state owned by the controller.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Motion {
    /// Current velocity in world space.
    pub velocity: Vec3,
    /// Whether walkable ground was touched on the last step.
    pub grounded: bool,
    /// Remaining seconds in which a jump is still allowed.
    pub coyote: f32,
}

/// Services the controller needs besides the player query.
#[derive(SystemParam)]
struct Env<'w, 's> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    settings: Res<'w, Persistent<Settings>>,
    slide: MoveAndSlide<'w, 's>,
}

/// Returns the horizontal velocity for this step.
fn horizontal(motion: &Motion, intent: &Intent, swimming: bool, dt: f32) -> Vec3 {
    let base = if intent.sprint {
        SPRINT_SPEED
    } else {
        WALK_SPEED
    };
    let speed = if swimming { base * SWIM_FACTOR } else { base };
    let rate = if motion.grounded && !swimming {
        GROUND_ACCEL
    } else {
        AIR_ACCEL
    };
    let current = Vec3::new(motion.velocity.x, 0.0, motion.velocity.z);
    approach(current, intent.wish * speed, rate, dt)
}

/// Runs one step of input, gravity or buoyancy, and collision-aware movement.
fn drive(env: Env, player: Single<(Entity, &mut Transform, &mut Motion, &Collider), With<Player>>) {
    let (entity, mut transform, mut motion, collider) = player.into_inner();
    let dt = env.time.delta_secs().min(MAX_DT);
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let intent = read_intent(&env.keys, &env.settings.controls.keys, yaw);
    let swimming = swim::is_swimming(transform.translation.y);
    let flat = horizontal(&motion, &intent, swimming, dt);
    let vertical = if swimming {
        swim::buoyancy_velocity(transform.translation.y, intent.jump)
    } else {
        land_vertical(&mut motion, &intent, dt)
    };
    let mut grounded = false;
    let out = env.slide.move_and_slide(
        collider,
        transform.translation,
        transform.rotation,
        flat + Vec3::Y * vertical,
        Duration::from_secs_f32(dt),
        &MoveAndSlideConfig::default(),
        &SpatialQueryFilter::from_excluded_entities([entity]),
        |hit| {
            grounded |= !hit.intersects() && hit.normal.y > MIN_GROUND_NORMAL;
            MoveAndSlideHitResponse::Accept
        },
    );
    transform.translation = out.position;
    motion.velocity = out.projected_velocity;
    motion.grounded = grounded && vertical <= 0.0;
}

/// Registers the controller system.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, drive.run_if(playing));
}
