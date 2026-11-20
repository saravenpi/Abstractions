//! The outline of the fuselage and of the canopy: the stations the hull is lofted through. The
//! parts that grow out of it, the wings, the fin and the engines, are in their own files.
//! The fuselage and the canopy that are drawn are open over the cockpit: see `shell`.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::ship::solid::Solid;

/// Sides of the fuselage.
pub(super) const SIDES: u32 = 8;

/// Stations of the fuselage from nose to tail: z, half width, half height and height of the
/// centre. The three of the cabin, at [`PRE`], [`JOINT`] and [`POST`], are the ones the cockpit
/// is fitted to and do not change: the nose lies in front of them and the tail behind.
pub(super) const FUSELAGE: [[f32; 4]; 8] = [
    [-3.95, 0.06, 0.045, -0.12],
    [-3.55, 0.18, 0.1, -0.11],
    [-3.2, 0.42, 0.3, -0.05],
    [-1.8, 0.85, 0.55, 0.05],
    [0.4, 1.0, 0.65, 0.1],
    [1.5, 1.0, 0.64, 0.12],
    [2.4, 0.9, 0.6, 0.15],
    [3.2, 0.6, 0.45, 0.2],
];

/// Index in [`FUSELAGE`] of the station in front of the mouth of the cockpit.
pub(super) const PRE: usize = 2;

/// Index in [`FUSELAGE`] of the joint between the nose and the body, in the middle of the mouth.
pub(super) const JOINT: usize = 3;

/// Index in [`FUSELAGE`] of the station behind the mouth of the cockpit.
pub(super) const POST: usize = 4;

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
    lathe(&mut body, &FUSELAGE[JOINT..], (joint, tail), SIDES);
    lathe(&mut nose, &FUSELAGE[..=JOINT], (tip, joint), SIDES);
    [body, nose]
}

/// Returns the outline of the canopy as a closed bubble. Its rim and both of its ends are
/// inside the fuselage. The glass that is drawn is the part of it above the hull.
#[cfg(test)]
pub fn canopy() -> Solid {
    let mut glass = Solid::default();
    lathe(&mut glass, &CANOPY, CANOPY_ENDS, CANOPY_SIDES);
    glass
}
