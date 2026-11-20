//! The landing gear: a leg with an oleo strut, a piston, a collar, a drag brace and a round pad,
//! and the three pods that hold the legs when they fold. A leg hangs from its hinge at the origin
//! and is scaled along y to fold: its top end, and the top end of its brace, stay in the hinge.

use bevy::prelude::*;

use super::loft::{Section, column, solid};
use crate::ship::gear::STRUT;
use crate::ship::solid::Solid;

/// Sides of the round parts of a leg.
const SIDES: u32 = 8;

/// Stations of a pod from its front: z, half width, height of the top and of the bottom. The
/// bottom is the dark bay, two doors with a seam between them.
pub(super) const MAIN: [[f32; 4]; 4] = [
    [0.85, 0.12, -0.2, -0.4],
    [1.2, 0.31, -0.12, -0.56],
    [1.8, 0.31, -0.12, -0.56],
    [2.15, 0.12, -0.2, -0.42],
];

/// Stations of the pod of the nose gear, like [`MAIN`].
pub(super) const NOSE: [[f32; 4]; 4] = [
    [-2.95, 0.08, -0.25, -0.33],
    [-2.6, 0.2, -0.2, -0.56],
    [-2.2, 0.2, -0.2, -0.56],
    [-1.85, 0.1, -0.3, -0.46],
];

/// The tones of the nine panels of a pod: the top, the sides and the dark doors.
const TONES: [u8; 9] = [0, 0, 2, 2, 4, 4, 2, 2, 0];

/// Returns the section of a pod at a station, centred `x` metres from the middle.
fn section(x: f32, [z, half, top, bottom]: [f32; 4]) -> Section {
    let c = (half * 0.5).min(0.08);
    let at = |dx: f32, y: f32| Vec3::new(x + dx, y, z);
    vec![
        at(-half + c, top),
        at(half - c, top),
        at(half, top - c),
        at(half, bottom + c),
        at(half - c, bottom),
        at(0.0, bottom + 0.012),
        at(-half + c, bottom),
        at(-half, bottom + c),
        at(-half, top - c),
    ]
}

/// Returns the pod of the stations `stations`, `x` metres from the middle.
pub fn pod(x: f32, stations: &[[f32; 4]]) -> Solid {
    let sections: Vec<Section> = stations.iter().map(|&at| section(x, at)).collect();
    let mut pod = Solid::default();
    solid(&mut pod, &sections, |_, panel| TONES[panel]);
    pod
}

/// Returns the corner points of the pod of `stations` at `x`, for the tests.
pub fn corners(x: f32, stations: &[[f32; 4]]) -> Vec<Vec3> {
    stations.iter().flat_map(|&at| section(x, at)).collect()
}

/// Returns the drag brace: a flat rod from the strut, 0.6 m down, up and back to the hinge.
fn brace() -> Solid {
    let (low, high) = (Vec3::new(0.0, -0.6, 0.04), Vec3::new(0.0, -0.05, 0.4));
    let wide = Vec3::X * 0.03;
    let thick = (high - low).cross(Vec3::X).normalize() * 0.05;
    let mut brace = Solid::default();
    brace.plate([low - wide, low + wide, high + wide, high - wide], thick);
    brace
}

/// Returns one leg of the gear, hanging from its hinge at the origin, its pad flat on the
/// foot [`STRUT`] metres down.
pub fn leg() -> Solid {
    let mut leg = Solid::default();
    column(
        &mut leg,
        Vec3::ZERO,
        &[(0.0, 0.075), (-0.55, 0.075)],
        (SIDES, 0),
    );
    column(
        &mut leg,
        Vec3::ZERO,
        &[(-0.5, 0.1), (-0.6, 0.1)],
        (SIDES, 0),
    );
    column(
        &mut leg,
        Vec3::ZERO,
        &[(-0.55, 0.05), (-1.02, 0.05)],
        (SIDES, 0),
    );
    let pad = [(-0.98, 0.1), (-1.03, 0.3), (-STRUT as f32, 0.3)];
    column(&mut leg, Vec3::ZERO, &pad, (SIDES, 0));
    leg.merge(&brace(), Vec3::ZERO);
    leg
}
