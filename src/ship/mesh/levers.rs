//! The parts of the cockpit that move: the stick and the throttle lever. Each one is built
//! around its hinge at the origin and is turned there.

use bevy::prelude::*;

use super::frame::lathe;
use crate::ship::cockpit::{LEVER_LENGTH, STICK_LENGTH};
use crate::ship::solid::Solid;

/// Length of the grip of the stick and of the handle of the throttle lever.
const GRIP: (f32, f32) = (0.1, 0.05);

/// Builds the centre stick, standing on its hinge: a thin shaft and a thicker grip.
pub fn stick_mesh() -> Mesh {
    let (top, shaft) = (STICK_LENGTH.0, STICK_LENGTH.0 - GRIP.0);
    let ring = |y: f32, r: f32| [y, r, r, 0.0];
    let mut upright = Solid::default();
    let outline = [
        ring(0.0, 0.012),
        ring(shaft, 0.012),
        ring(shaft, 0.024),
        ring(top, 0.02),
    ];
    lathe(&mut upright, &outline, (Vec3::ZERO, Vec3::Z * top), 6);
    stood(upright)
}

/// Builds the throttle lever, standing on its hinge: a flat shaft and a handle across it.
pub fn lever_mesh() -> Mesh {
    let mut lever = Solid::default();
    let (top, shaft) = (LEVER_LENGTH.0, LEVER_LENGTH.0 - GRIP.1);
    lever.slab(
        Vec3::new(-0.008, 0.0, -0.012),
        Vec3::new(0.008, shaft, 0.012),
    );
    lever.slab(Vec3::new(-0.035, shaft, -0.02), Vec3::new(0.035, top, 0.02));
    lever.into_mesh()
}

/// Returns the mesh of a part lathed along z, stood up along y.
fn stood(part: Solid) -> Mesh {
    part.into_mesh()
        .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
}
