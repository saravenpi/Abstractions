//! The landing gear: a leg with a sleeve, an oleo piston, a torque link, a drag brace and a long
//! skid foot, and the three pods that hold the legs when they fold. A leg hangs from its hinge at the origin
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
    [-2.95, 0.08, -0.25, -0.36],
    [-2.6, 0.2, -0.2, -0.56],
    [-2.2, 0.2, -0.2, -0.56],
    [-1.85, 0.1, -0.3, -0.46],
];

/// The tones of the nine panels of a pod: the top, the sides and the dark doors.
const TONES: [u8; 9] = [0, 0, 2, 2, 4, 4, 2, 2, 0];

/// Returns the section of a pod at a station, centred `x` metres from the middle.
fn section(x: f32, [z, half, top, bottom]: [f32; 4]) -> Section {
    let c = (half * 0.5).min(0.12);
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

/// Returns the torque link in front of the piston: two flat arms from the sleeve to a knee and
/// back to the ankle, which keep the foot from turning.
fn link() -> Solid {
    let knee = [
        Vec3::new(0.0, -0.4, -0.07),
        Vec3::new(0.0, -0.7, -0.24),
        Vec3::new(0.0, -0.99, -0.07),
    ];
    let wide = Vec3::X * 0.035;
    let mut link = Solid::default();
    for pair in knee.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let thick = (b - a).cross(Vec3::X).normalize() * 0.03;
        link.plate([a - wide, a + wide, b + wide, b - wide], thick);
    }
    link
}

/// Returns the foot: a long skid plate with a raised toe and heel, flat on the ground [`STRUT`]
/// metres under the hinge. Folded, it lies under its pod and closes the bay.
fn foot() -> Solid {
    let low = -STRUT as f32;
    let section = |z: f32, half: f32, top: f32, bottom: f32| -> Section {
        [(-half, top), (half, top), (half, bottom), (-half, bottom)]
            .map(|(x, y)| Vec3::new(x, low + y, z))
            .into()
    };
    let sections = [
        section(-0.38, 0.07, 0.1, 0.07),
        section(-0.26, 0.13, 0.06, 0.0),
        section(0.2, 0.13, 0.06, 0.0),
        section(0.3, 0.09, 0.08, 0.04),
    ];
    let mut foot = Solid::default();
    solid(&mut foot, &sections, |_, _| 0);
    foot.slab(
        Vec3::new(-0.07, low + 0.05, -0.1),
        Vec3::new(0.07, low + 0.16, 0.1),
    );
    foot
}

/// Returns one leg of the gear, hanging from its hinge at the origin: the sleeve with its
/// collar, the piston of the oleo, the torque link, the drag brace and the foot.
pub fn leg() -> Solid {
    let mut leg = Solid::default();
    let rings: [&[(f32, f32)]; 3] = [
        &[(0.0, 0.09), (-0.44, 0.09)],
        &[(-0.38, 0.115), (-0.47, 0.115)],
        &[(-0.42, 0.05), (-1.0, 0.05)],
    ];
    for ring in rings {
        column(&mut leg, Vec3::ZERO, ring, (SIDES, 0));
    }
    for part in [foot(), link(), brace()] {
        leg.merge(&part, Vec3::ZERO);
    }
    leg
}
