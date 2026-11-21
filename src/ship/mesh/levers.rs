//! The parts of the cockpit that move: the stick and the throttle lever. Each one is built
//! around its hinge at the origin and is turned there. The shafts are dark and the grips are
//! the accent colour, so the hands of the pilot are easy to find.

use bevy::prelude::*;

use super::loft::column;
use crate::ship::cockpit::{LEVER_LENGTH, STICK_LENGTH};
use crate::ship::solid::Solid;

/// Sides of the round parts of the stick.
const SIDES: u32 = 8;

/// Builds the side stick, standing on its hinge: a boot, a thin shaft, a grip that swells
/// under the hand, and a dark hat switch on top.
pub fn stick_mesh() -> Mesh {
    let top = STICK_LENGTH.0;
    let mut stick = Solid::default();
    column(&mut stick, Vec3::ZERO, &[(0.05, 0.018), (0.0, 0.04)], (SIDES, 3));
    column(&mut stick, Vec3::ZERO, &[(0.15, 0.011), (0.03, 0.011)], (SIDES, 3));
    let grip = [(top, 0.017), (top - 0.02, 0.024), (0.16, 0.026), (0.14, 0.015)];
    column(&mut stick, Vec3::ZERO, &grip, (SIDES, 0));
    stick.set_tone(3);
    stick.slab(Vec3::new(-0.008, top - 0.004, -0.008), Vec3::new(0.008, top + 0.012, 0.008));
    stick.into_mesh()
}

/// Builds the throttle lever, standing on its hinge: a flat dark arm and a handle across it.
pub fn lever_mesh() -> Mesh {
    let mut lever = Solid::default();
    let (top, arm) = (LEVER_LENGTH.0, LEVER_LENGTH.0 - 0.05);
    lever.set_tone(3);
    lever.slab(Vec3::new(-0.008, 0.0, -0.012), Vec3::new(0.008, arm + 0.01, 0.012));
    lever.set_tone(0);
    lever.slab(Vec3::new(-0.04, arm, -0.018), Vec3::new(0.04, top, 0.018));
    lever.into_mesh()
}
