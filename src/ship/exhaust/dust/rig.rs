//! Shared setup of the dust tests: a smooth test planet, a puff over it and an anchor on it.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::puff::Mote;
use crate::space::{Anchor, Frame};
use crate::universe::Universe;

/// Sea level radius of the test planet in metres.
pub const RADIUS: f64 = 1.0e6;

/// Returns a puff `height` metres over the ground of the test planet in direction `dir`,
/// carried by `flow` and settling at `settle`.
pub fn aloft(dir: DVec3, height: f64, (flow, settle): (Vec3, f32)) -> Mote {
    let up = dir.normalize();
    Mote {
        at: up * (RADIUS + height),
        flow,
        settle,
        floor: RADIUS,
        read: up * (RADIUS + height),
        life: 100.0,
        size: 1.0,
        ..Mote::default()
    }
}

/// Returns the anchor of the surface of the test planet whose origin is the sea level point in
/// direction `up`, with render +Y up there.
pub fn anchored(up: DVec3) -> Anchor {
    Anchor {
        frame: Frame::Surface(Universe::new(4).home()),
        origin: up * RADIUS,
        rotation: DQuat::from_rotation_arc(up, DVec3::Y),
        flat: false,
    }
}
