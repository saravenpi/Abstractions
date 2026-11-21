//! The flight gear inside the controller: what it is told each frame and how it moves the body.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::controller::Motion;
use super::ground::{Ground, slope_velocity, snap_velocity};
use super::intent::Intent;
use super::{BODY_LENGTH, BODY_RADIUS};
use crate::gear::{Gear, GearInput, Mode, air_density};
use crate::universe::heights;
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
    /// Heading and pitch of the view.
    pub(super) aim: Vec2,
    /// Gravity in metres per second squared.
    pub(super) gravity: f32,
    /// Density of the liquid of the sea in kilograms per cubic metre, 0 when frozen or absent.
    pub(super) liquid: f32,
}

/// Returns the air density `height` metres above the sea of `globe`.
pub(super) fn density(globe: &Globe, height: f32) -> f32 {
    let gravity = f64::from(globe.setting().gravity);
    let [top, ..] = heights(globe.record(), gravity, globe.radius());
    air_density(top as f32, height)
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

/// Frames in a row a body the gear carries may rest on walkable ground, slower than [`STILL`],
/// before it counts as landed whatever the gap under it: a tenth of a second.
pub(super) const WATCH: u8 = 6;

/// Speed in metres per second under which a body the gear carries over walkable ground is at
/// rest on it rather than skimming it, as a take-off run does.
pub(super) const STILL: f32 = 3.0;

/// Returns true when the flight gear should treat `floor` as ground under a body in `mode`
/// that has rested `rested` frames on it, see [`aground`].
///
/// A body the gear carries through the air lands once it really touches, or once it has rested
/// on the ground for [`WATCH`] frames: no state stays falling on the ground. One that walks,
/// slides or rolls is on the ground wherever the walking snap holds it.
pub(super) fn touching(mode: Mode, floor: Option<Ground>, rested: u8) -> bool {
    floor.is_some_and(|ground| !mode.airborne() || ground.gap <= TOUCH || rested >= WATCH)
}

/// Returns the frames a body has rested on the ground in the air, `rested` before this one:
/// one more while the gear carries it in `mode` over walkable `floor` slower than [`STILL`]
/// metres per second, else none.
pub(super) fn aground(rested: u8, mode: Mode, floor: Option<Ground>, speed: f32) -> u8 {
    match mode.airborne() && floor.is_some() && speed < STILL {
        true => rested.saturating_add(1),
        false => 0,
    }
}

/// Ground speed in metres per second above which a body with its wings folded that runs away
/// from the view heads where it goes.
const HEADED: f32 = 2.0;

/// Returns the heading and pitch that aim the gear of a body moving as `motion`, the view being
/// `view`. It is the view, so the wings open, and open again in a fall, toward where the player
/// looks; the path then turns onto it, see `gear::assist`. Only a folded body that moves away
/// from the view, more than a quarter turn off it, as on a run toward the camera, heads where
/// it goes: wings opened with the nose against the path would turn it over backward.
fn aimed(motion: &Motion, view: Vec2) -> Vec2 {
    let flat = motion.velocity.with_y(0.0);
    let going = (-flat.x).atan2(-flat.z);
    let away = super::wrap(going - view.x).abs() > std::f32::consts::FRAC_PI_2;
    match !motion.wings && flat.length() > HEADED && away {
        true => Vec2::new(going, view.y),
        false => view,
    }
}

/// Returns what the gear is told this frame.
///
/// The run key held in the air fires the boots: a brake with the wingsuit folded, the boost with
/// it asked open. The forward key dives and the back key pulls, the strafe keys roll and the
/// yaw keys turn flat, all at a rate in the frame of the body. On the wings the view aims the body only while the
/// steer button is held; folded it always does.
pub(super) fn input(motion: &Motion, intent: &Intent, air: &Air, on: (bool, bool)) -> GearInput {
    GearInput {
        deploy: motion.wings && motion.sling <= 0.0,
        boost: motion.boots,
        pitch: -intent.forward,
        roll: intent.right,
        yaw: intent.yaw,
        look: air.look,
        aim: (!motion.wings || intent.steer).then_some(aimed(motion, air.aim)),
        grounded: on.0,
        in_water: on.1,
        liquid: air.liquid,
        density: air.density,
        air: 1.0,
        gravity: air.gravity,
        clearance: air.clearance,
        assist: 1.0,
        wind: air.wind,
        boots: 1.0,
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
