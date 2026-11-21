//! The wings: one aerofoil section lofted through four stations, from a thick root inside the
//! fuselage, along a strake that leaves the chine, out to a thin swept tip.
//! The top is flat over the middle of the chord and falls a little toward the tip; the rear
//! quarter is the flap, set off from the wing by a shallow step and a second tone.

use bevy::prelude::*;

use super::loft::{Section, solid};
use crate::ship::solid::Solid;

/// Stations of the right wing from the root to the tip: x, z of the leading edge, chord and
/// thickness, in metres. The root is inside the fuselage. From it a long strake runs out to the
/// crank, then a swept panel to a short tip; the trailing edge is swept forward. The span is
/// 6.4 m across the tips.
pub(super) const STATIONS: [[f32; 4]; 4] = [
    [0.45, -1.3, 3.8, 0.26],
    [1.25, -0.1, 2.85, 0.3],
    [1.9, 0.5, 2.15, 0.26],
    [3.2, 1.7, 0.72, 0.09],
];

/// Height of the flat top of the wing at `x` metres from the middle: 0.04 m at the root, falling
/// to -0.16 m at the tip, 3.2 m out.
pub fn top(x: f32) -> f32 {
    0.04 - 0.2 * (x - 0.45) / 2.75
}

/// The section of the wing: the share of the chord from the leading edge, and the share of the
/// thickness under the top, of each corner, from the leading edge over the top to the flap,
/// the trailing edge and back under the wing.
const SECTION: [(f32, f32); 10] = [
    (0.0, 0.5),
    (0.07, 0.15),
    (0.25, 0.0),
    (0.72, 0.0),
    (0.74, 0.05),
    (1.0, 0.68),
    (1.0, 0.86),
    (0.72, 1.0),
    (0.3, 1.0),
    (0.06, 0.86),
];

/// The tone of each panel of the section: the top, the dark seam of the flap, the flap, and the
/// grey underside.
const TONES: [u8; 10] = [0, 0, 0, 3, 1, 2, 2, 2, 2, 2];

/// Returns the section of the wing at `station`, on the right for `side` 1 and on the left for -1.
fn section(side: f32, [x, front, chord, thick]: [f32; 4]) -> Section {
    let corner = |&(along, down): &(f32, f32)| {
        Vec3::new(x * side, top(x) - down * thick, front + along * chord)
    };
    SECTION.iter().map(corner).collect()
}

/// Returns the wing of one side: -1 for the left, 1 for the right.
pub fn wing(side: f32) -> Solid {
    let sections: Vec<Section> = STATIONS.iter().map(|&at| section(side, at)).collect();
    let mut wing = Solid::default();
    solid(&mut wing, &sections, |_, panel| TONES[panel]);
    wing
}

/// Returns the corner points of the wing of one side, for its collider.
pub fn corners(side: f32) -> Vec<Vec3> {
    STATIONS.iter().flat_map(|&at| section(side, at)).collect()
}
