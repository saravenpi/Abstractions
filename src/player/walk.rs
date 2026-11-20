//! Walking, jumping and swimming, and the hand-over to the flight gear in the air.

use bevy::prelude::*;

use super::controller::Motion;
use super::crouch;
use super::dive::{SwimInput, swim_velocity, swim_wish};
use super::flight::{Air, carried, input, touching};
use super::ground::{Ground, RISING_SPEED, slope_velocity, snap_velocity, walkable};
use super::intent::{Intent, approach, land_vertical};
use super::lead::View;
use super::swim;
use crate::gear::{Gear, GearInput};

/// Walking speed in meters per second.
const WALK_SPEED: f32 = 5.0;

/// Sprinting speed in meters per second.
const SPRINT_SPEED: f32 = 8.5;

/// Horizontal acceleration rate on the ground.
const GROUND_ACCEL: f32 = 14.0;

/// Horizontal acceleration rate in the air.
const AIR_ACCEL: f32 = 3.5;

/// Horizontal speed below which an idle body stops dead.
const REST_SPEED: f32 = 0.05;

/// Per-frame values shared by the movement steps.
pub(super) struct Step {
    /// Simulated seconds.
    pub(super) dt: f32,
    /// Seconds since startup, for the floating bob.
    pub(super) time: f32,
    /// Where the player looks, with the turn of this frame already in.
    pub(super) view: View,
    /// Altitude of the body center above sea level, which the air thins with.
    pub(super) height: f32,
    /// Height of the body center above the liquid surface, infinite on a dry world.
    pub(super) afloat: f32,
    /// Gravity at the body in meters per second squared.
    pub(super) gravity: f32,
    /// How thick the air of the planet is against the home planet, 0 without air.
    pub(super) air: f32,
}

/// Returns the horizontal velocity for this step.
fn horizontal(motion: &Motion, intent: &Intent, step: &Step) -> Vec3 {
    let base = if intent.sprint && !intent.crouch && motion.crouch <= 0.0 {
        SPRINT_SPEED
    } else {
        WALK_SPEED * crouch::pace(motion.crouch)
    };
    let rate = if motion.grounded {
        GROUND_ACCEL
    } else {
        AIR_ACCEL
    };
    let current = Vec3::new(motion.velocity.x, 0.0, motion.velocity.z);
    let next = approach(
        current,
        intent.wish * base * swim::wade_factor(step.afloat),
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

/// Returns the swimming or walking velocity for this frame, as the swim state says.
fn velocity_for(motion: &mut Motion, intent: &Intent, floor: Option<Ground>, step: &Step) -> Vec3 {
    if !motion.swimming {
        let ground = walkable(floor, motion.velocity.y > RISING_SPEED);
        return walk_step(motion, intent, ground, step);
    }
    motion.grounded = false;
    let input = SwimInput {
        wish: swim_wish(
            intent.forward,
            intent.right,
            motion.steer.yaw,
            step.view.pitch,
        ),
        sprint: intent.sprint,
        up: intent.jump,
    };
    swim_velocity(motion.velocity, &input, step.afloat, step.time, step.dt)
}

/// Steps the flight gear and returns the velocity to move the body by: the gear's while it
/// carries the body, else the walking or swimming one.
pub(super) fn velocity(
    (motion, gear): (&mut Motion, &mut Gear),
    intent: &Intent,
    floor: Option<Ground>,
    air: &Air,
    step: &Step,
) -> Vec3 {
    let gap = walkable(floor, false).map(|ground| ground.gap);
    let ground = walkable(floor, motion.velocity.y > RISING_SPEED);
    motion.swimming = swim::next_swimming(motion.swimming, step.afloat, gap);
    let down = intent.crouch && crouch::allowed(motion, gear);
    motion.crouch = crouch::settle(motion.crouch, down, step.dt);
    let on = (touching(gear.state.mode, ground), motion.swimming);
    let told = GearInput {
        air: step.air,
        ..input(motion, intent, air, on)
    };
    gear.drive(motion.velocity, told, step.dt);
    motion.spent = intent.jump && (motion.spent || gear.state.mode.airborne());
    let Some(velocity) = carried(gear, ground, step.dt) else {
        motion.wings &= !(on.0 || on.1);
        let hop = Intent {
            jump: intent.jump && (!motion.spent || motion.swimming),
            ..*intent
        };
        return velocity_for(motion, &hop, floor, step);
    };
    motion.grounded = !gear.state.mode.airborne();
    velocity
}
