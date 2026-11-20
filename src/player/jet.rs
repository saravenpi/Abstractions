//! The boot water jet inside the controller: diving and climbing along the view, skimming at
//! the surface.

use bevy::prelude::*;

use super::dive::{HOP_DEPTH, float_speed};
use super::walk::Step;
use crate::gear::{Gear, jet};

/// Sine of the angle below the horizon past which the view dives a body jetting at the surface
/// (about 17 degrees). Above it the body skims along the surface and floats.
const DIVE_SINE: f32 = 0.3;

/// Returns the velocity of a swimming body whose boots jet, from `current`, in its upright frame.
///
/// Under the surface the jet pushes along the view, so the body dives, climbs and turns with
/// the camera. At the surface a view that does not dive skims: the jet pushes along the flat
/// heading of the view and the body floats on the water.
pub(super) fn jet_velocity(current: Vec3, gear: &Gear, step: &Step) -> Vec3 {
    let look = step.view.direction();
    let skims = step.afloat > -HOP_DEPTH && look.y > -DIVE_SINE;
    let aim = match skims {
        true => look.with_y(0.0).normalize_or(Vec3::NEG_Z),
        false => look,
    };
    let pushed = jet(current, aim, &gear.state, &gear.input, step.dt);
    match skims {
        true => pushed.with_y(float_speed(current.y, step.afloat, step.time, step.dt)),
        false => pushed,
    }
}
