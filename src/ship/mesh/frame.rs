//! The outline of the fuselage and of the canopy: the stations the hull is lofted through. The
//! parts that grow out of it, the wings, the fin and the engines, are in their own files.
//! The fuselage that is drawn is open over the cockpit: see `shell`. The canopy and the spine
//! behind it are drawn by `dome`.

use bevy::prelude::*;

#[cfg(test)]
use crate::ship::solid::Solid;

/// Corners of a section of the fuselage.
pub(super) const SIDES: u32 = 8;

/// The section of the fuselage: where each of its eight corners is, as shares of the half width
/// and of the half height of a station, from the right chine round the back, the left chine and
/// the belly. The chines are the widest corners, a quarter of the way down; above them the sides
/// lean in to a narrow back, and the belly is nearly flat.
const PROFILE: [(f32, f32); 8] = [
    (1.0, -0.25),
    (0.6, 0.7),
    (0.0, 1.0),
    (-0.6, 0.7),
    (-1.0, -0.25),
    (-0.62, -0.92),
    (0.0, -1.0),
    (0.62, -0.92),
];

/// Stations of the fuselage from nose to tail: z, half width at the chine, half height and
/// height of the centre. The three of the cabin, at [`PRE`], [`JOINT`] and [`POST`], are the
/// ones the cockpit is fitted to: the spear of the nose lies in front of them and the tail behind.
pub(super) const FUSELAGE: [[f32; 4]; 8] = [
    [-4.3, 0.07, 0.035, -0.165],
    [-3.85, 0.2, 0.09, -0.14],
    [-3.2, 0.42, 0.3, -0.05],
    [-1.8, 0.85, 0.55, 0.05],
    [0.4, 1.0, 0.65, 0.1],
    [1.5, 1.0, 0.58, 0.14],
    [2.4, 0.88, 0.48, 0.19],
    [3.2, 0.56, 0.34, 0.22],
];

/// Index in [`FUSELAGE`] of the station in front of the mouth of the cockpit.
pub(super) const PRE: usize = 2;

/// Index in [`FUSELAGE`] of the joint between the nose and the body, in the middle of the mouth.
pub(super) const JOINT: usize = 3;

/// Index in [`FUSELAGE`] of the station behind the mouth of the cockpit.
pub(super) const POST: usize = 4;

/// The tip of the nose, the middle of the joint between the nose and the body, and the end of the tail.
pub(super) const SPINE: [Vec3; 3] = [
    Vec3::new(0.0, -0.17, -4.45),
    Vec3::new(0.0, 0.05, -1.8),
    Vec3::new(0.0, 0.22, 3.3),
];

/// Stations of the canopy and of the spine behind it, from the windscreen back: z, half width of
/// the foot, half width of the roof and height of the roof. The foot of each is just under the
/// skin of the hull. The first [`GLAZED`] are the glass, which lifts; the rest is the spine.
pub(super) const CANOPY: [[f32; 4]; 6] = [
    [-2.3, 0.36, 0.18, 0.86],
    [-1.45, 0.5, 0.3, 1.28],
    [-0.85, 0.5, 0.28, 1.27],
    [-0.6, 0.48, 0.26, 1.23],
    [0.4, 0.32, 0.14, 1.02],
    [1.5, 0.17, 0.06, 0.95],
];

/// How many stations of [`CANOPY`] carry glass: the last of them is the hoop the canopy hinges on.
pub(super) const GLAZED: usize = 3;

/// Where the canopy and the spine end in the hull: the foot of the windscreen, and the end of the
/// spine in the root of the fin.
pub(super) const CANOPY_ENDS: (Vec3, Vec3) = (Vec3::new(0.0, 0.27, -3.0), Vec3::new(0.0, 0.85, 2.3));

/// How far under the skin of the hull the foot of the canopy and of the spine goes, in metres.
pub(super) const SUNK: f32 = 0.025;

/// Returns the corner `i` of the section of the fuselage at a station.
pub(super) fn rim([z, w, h, y]: [f32; 4], i: u32) -> Vec3 {
    let (across, up) = PROFILE[(i % SIDES) as usize];
    Vec3::new(across * w, y + up * h, z)
}

/// Adds a closed body lathed along z through `stations`, whose corner `i` is `corner(station, i)`,
/// closed by fans to the two `ends`: the first in front of the stations, the second behind them.
#[cfg(test)]
pub fn lathe(
    solid: &mut Solid,
    stations: &[[f32; 4]],
    ends: (Vec3, Vec3),
    corner: impl Fn([f32; 4], u32) -> Vec3,
    sides: u32,
) {
    let (first, last) = (stations[0], stations[stations.len() - 1]);
    let unit = [0.0, 1.0, 1.0, 0.0];
    for i in 0..sides {
        let out = corner(unit, i) + corner(unit, i + 1);
        solid.tri([ends.0, corner(first, i), corner(first, i + 1)], Vec3::NEG_Z);
        solid.tri([ends.1, corner(last, i), corner(last, i + 1)], Vec3::Z);
        for pair in stations.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let corners = [corner(a, i), corner(a, i + 1), corner(b, i + 1), corner(b, i)];
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
    lathe(&mut body, &FUSELAGE[JOINT..], (joint, tail), rim, SIDES);
    lathe(&mut nose, &FUSELAGE[..=JOINT], (tip, joint), rim, SIDES);
    [body, nose]
}
