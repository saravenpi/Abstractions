//! The flight gear inside the controller: what it is told each frame and how it moves the body.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::controller::Motion;
use super::ground::{Ground, slope_velocity, snap_velocity};
use super::intent::Intent;
use super::{BODY_LENGTH, BODY_RADIUS};
use crate::gear::{Gear, GearInput, Mode, air_density};
use crate::universe::air;
use crate::world::Globe;

/// Reach of the ray that measures the free height under the body, in metres: two seconds of
/// the fastest dive, which is as far ahead as the ground assist looks.
const CLEARANCE_REACH: f32 = 200.0;

/// Gap to walkable ground under which a flying body counts as touching down, in metres. It is
/// tighter than the reach of the walking snap, so that a take-off run skims the ground freely.
pub(super) const TOUCH: f32 = 0.08;

/// The air and the ground around the body this frame, in its upright frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Air {
    /// Air density in kilograms per cubic metre.
    pub(super) density: f32,
    /// Velocity of the wind over the ground.
    pub(super) wind: Vec3,
    /// Free height under the soles in metres, infinite when nothing is in reach.
    pub(super) clearance: f32,
    /// Unit direction the camera looks along.
    pub(super) look: Vec3,
    /// Heading and pitch of the view. The pitch passes the vertical in a loop.
    pub(super) aim: Vec2,
    /// Gravity in metres per second squared.
    pub(super) gravity: f32,
}

/// Returns the air density `height` metres above the sea of `globe`.
pub(super) fn density(globe: &Globe, height: f32) -> f32 {
    air_density(air(globe.radius())[0] as f32, height)
}

/// Returns the free height under the soles of a body centred at `at`, from a ray down the local
/// vertical. Call it in `Update`, where the spatial queries are valid.
pub(super) fn clearance(
    query: &SpatialQuery,
    at: Vec3,
    down: Vec3,
    filter: &SpatialQueryFilter,
) -> f32 {
    let half = BODY_LENGTH * 0.5 + BODY_RADIUS;
    let Ok(dir) = Dir3::new(down) else {
        return f32::INFINITY;
    };
    query
        .cast_ray(at, dir, CLEARANCE_REACH + half, true, filter)
        .map_or(f32::INFINITY, |hit| (hit.distance - half).max(0.0))
}

/// Returns true when the flight gear should treat `floor` as ground under a body in `mode`.
///
/// A body the gear carries through the air lands only once it really touches. One that
/// walks, slides or rolls is on the ground wherever the walking snap holds it.
pub(super) fn touching(mode: Mode, floor: Option<Ground>) -> bool {
    floor.is_some_and(|ground| !mode.airborne() || ground.gap <= TOUCH)
}

/// Returns what the gear is told this frame.
///
/// The run key held in the air fires the boots: a brake with the wingsuit folded, the boost with
/// it asked open. The forward key pushes the nose down and the back key pulls it up.
pub(super) fn input(motion: &Motion, intent: &Intent, air: &Air, on: (bool, bool)) -> GearInput {
    GearInput {
        deploy: motion.wings,
        boost: motion.boots || (motion.wings && intent.sprint),
        pitch: -intent.forward,
        roll: intent.right,
        look: air.look,
        aim: Some(air.aim),
        grounded: on.0,
        in_water: on.1,
        density: air.density,
        air: 1.0,
        gravity: air.gravity,
        clearance: air.clearance,
        assist: 1.0,
        wind: air.wind,
    }
}

/// Returns the velocity the gear moves the body by this frame, or nothing while the gear is
/// off. A body that slides or rolls is glued to the ground the way a walking one is.
pub(super) fn carried(gear: &Gear, floor: Option<Ground>, dt: f32) -> Option<Vec3> {
    let state = &gear.state;
    match (state.mode, floor) {
        (Mode::Off, _) => None,
        (Mode::Slide | Mode::Roll, Some(ground)) => {
            let flat = state.velocity.with_y(0.0);
            Some(slope_velocity(flat, ground.normal) + Vec3::Y * snap_velocity(ground.gap, dt))
        }
        _ => Some(state.velocity),
    }
}

/// Hands the velocity the physics left back to the gear and returns the velocity the body
/// keeps. On the ground nothing of the snap is kept.
pub(super) fn settle(gear: &mut Gear, moved: Vec3) -> Vec3 {
    let kept = match gear.state.mode.airborne() {
        true => moved,
        false => moved.with_y(0.0),
    };
    gear.collide(kept);
    gear.state.velocity
}
