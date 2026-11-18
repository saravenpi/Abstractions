//! The airframe of the ship: the outline of the fuselage and of the canopy, the wings and the
//! fin. Every part goes into the part that carries it, so that no plate hangs beside the hull.
//! The fuselage and the canopy that are drawn are open over the cockpit: see `shell`.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::ship::solid::Solid;

/// Sides of the fuselage.
pub(super) const SIDES: u32 = 8;

/// Stations of the fuselage from nose to tail: z, half width, half height and height of the centre.
pub(super) const FUSELAGE: [[f32; 4]; 5] = [
    [-3.2, 0.42, 0.3, -0.05],
    [-1.8, 0.85, 0.55, 0.05],
    [0.4, 1.0, 0.65, 0.1],
    [2.4, 0.9, 0.6, 0.15],
    [3.2, 0.6, 0.45, 0.2],
];

/// The tip of the nose, the middle of the joint between the nose and the body, and the end of the tail.
pub(super) const SPINE: [Vec3; 3] = [
    Vec3::new(0.0, -0.1, -4.2),
    Vec3::new(0.0, 0.05, -1.8),
    Vec3::new(0.0, 0.2, 3.3),
];

/// Stations of the canopy bubble, like [`FUSELAGE`].
pub(super) const CANOPY: [[f32; 4]; 2] = [[-2.1, 0.5, 0.42, 0.62], [-1.0, 0.58, 0.5, 0.72]];

/// Where the canopy bubble ends in the hull, in front and behind.
pub(super) const CANOPY_ENDS: (Vec3, Vec3) = (Vec3::new(0.0, 0.25, -2.9), Vec3::new(0.0, 0.6, 0.2));

/// Sides of the canopy bubble.
pub(super) const CANOPY_SIDES: u32 = 6;

/// Outline of the right wing from above: the root at the front and at the back, then the tip
/// at the back and at the front. The root is 0.2 m or more inside the fuselage.
const WING: [Vec3; 4] = [
    Vec3::new(0.45, -0.1, -0.2),
    Vec3::new(0.45, -0.1, 2.6),
    Vec3::new(3.2, -0.3, 2.2),
    Vec3::new(3.2, -0.3, 1.0),
];

/// Outline of the right winglet from the side, 0.05 m inside the end of the wing: it starts
/// under the wing and is longer than the wing end, which it takes in whole.
const WINGLET: [Vec3; 4] = [
    Vec3::new(3.15, -0.32, 0.85),
    Vec3::new(3.15, -0.32, 2.25),
    Vec3::new(3.15, 0.2, 2.4),
    Vec3::new(3.15, 0.2, 1.3),
];

/// Outline of the fin from the side. Its foot is 0.1 m or more under the roof of the fuselage.
const FIN: [Vec3; 4] = [
    Vec3::new(-0.05, 0.5, 1.5),
    Vec3::new(-0.05, 0.5, 3.15),
    Vec3::new(-0.05, 1.9, 3.4),
    Vec3::new(-0.05, 1.9, 2.7),
];

/// Returns the point of a station of a lathe along z at corner `i` of `sides`.
pub(super) fn ring([z, w, h, y]: [f32; 4], i: u32, sides: u32) -> Vec3 {
    let a = i as f32 / sides as f32 * TAU;
    Vec3::new(a.cos() * w, y + a.sin() * h, z)
}

/// Adds a closed body lathed along z through `stations`, closed by fans to the two `ends`: the
/// first in front of the stations, the second behind them.
pub fn lathe(solid: &mut Solid, stations: &[[f32; 4]], ends: (Vec3, Vec3), sides: u32) {
    let (first, last) = (stations[0], stations[stations.len() - 1]);
    for i in 0..sides {
        let out = ring([0.0, 1.0, 1.0, 0.0], i, sides) + ring([0.0, 1.0, 1.0, 0.0], i + 1, sides);
        solid.tri(
            [ends.0, ring(first, i, sides), ring(first, i + 1, sides)],
            Vec3::NEG_Z,
        );
        solid.tri(
            [ends.1, ring(last, i, sides), ring(last, i + 1, sides)],
            Vec3::Z,
        );
        for pair in stations.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let corners = [
                ring(a, i, sides),
                ring(a, i + 1, sides),
                ring(b, i + 1, sides),
                ring(b, i, sides),
            ];
            solid.quad(corners, out);
        }
    }
}

/// Returns the outline of the fuselage as two closed halves, the body and the nose: what the
/// fit tests hold every part against. The hull that is drawn is the same outline, open over
/// the cockpit.
#[cfg(test)]
pub fn fuselage() -> [Solid; 2] {
    let (mut body, mut nose) = (Solid::default(), Solid::default());
    let [tip, joint, tail] = SPINE;
    lathe(&mut body, &FUSELAGE[1..], (joint, tail), SIDES);
    lathe(&mut nose, &FUSELAGE[..2], (tip, joint), SIDES);
    [body, nose]
}

/// Returns the wing and the winglet of one side: -1 for the left, 1 for the right.
pub fn wing(side: f32) -> [Solid; 2] {
    let mirror = |p: Vec3| Vec3::new(p.x * side, p.y, p.z);
    let (mut wing, mut winglet) = (Solid::default(), Solid::default());
    wing.plate(WING.map(mirror), Vec3::Y * 0.14);
    winglet.plate(WINGLET.map(mirror), Vec3::X * 0.1 * side);
    [wing, winglet]
}

/// Returns the tail fin, which stands in the roof of the fuselage.
pub fn fin() -> Solid {
    let mut fin = Solid::default();
    fin.plate(FIN, Vec3::X * 0.1);
    fin
}

/// Returns the outline of the canopy as a closed bubble. Its rim and both of its ends are
/// inside the fuselage. The glass that is drawn is the part of it above the hull.
#[cfg(test)]
pub fn canopy() -> Solid {
    let mut glass = Solid::default();
    lathe(&mut glass, &CANOPY, CANOPY_ENDS, CANOPY_SIDES);
    glass
}
