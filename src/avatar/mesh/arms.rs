//! The arms, each hanging along -Y from its joint, written for the right side: a thin dark upper
//! arm under a shoulder plate hinged at the shoulder disc, a gauntlet that widens to the wrist,
//! and a hand. The left side is the mirror, with a larger shoulder plate.

use bevy::prelude::*;

use crate::avatar::tune::{ACCENT, DARK, FOREARM, SHADE, SHELL, UPPER_ARM};
use crate::skin::{Detail, Disc, LARGE, Plate, Ring, Section, Soup, discs, loft, plates};

/// The frame of the upper arm, from inside the shoulder to the elbow.
const UPPER: [Section; 2] = [
    Section::new(0.03, 0.036, 0.036),
    Section::new(-UPPER_ARM, 0.032, 0.032),
];

/// The housing of the elbow, and the shoulder disc with its bone cap on the outer face of the
/// shoulder plate.
const JOINTS: [Disc; 3] = [
    Disc::new([0.0, -UPPER_ARM, 0.0], Vec3::X, [0.046, 0.042], DARK)
        .sides(8)
        .fine(),
    Disc::new([0.086, -0.035, 0.0], Vec3::X, [0.05, 0.008], DARK).fine(),
    Disc::new([0.094, -0.035, 0.0], Vec3::X, [0.028, 0.004], SHELL)
        .sides(8)
        .fine(),
];

/// The shoulder plate of the right arm: a block over the joint and the top of the arm, flared
/// outward.
const SHORT: [Plate; 1] = [
    Plate::new([0.046, -0.05, 0.0], [0.042, 0.088, 0.08], [SHELL, SHELL])
        .turned(0.0, 0.0, 0.06)
        .cut(LARGE),
];

/// The shoulder plate of the left arm, written on the right: the block and a slate lame under
/// it, edged in the accent. It is the one difference between the two sides. The lame is short, so
/// the arm that holds the throttle stays inside the tub of the cockpit.
const LONG: [Plate; 2] = [
    SHORT[0],
    Plate::new([0.062, -0.152, 0.0], [0.012, 0.042, 0.068], [SHADE, ACCENT]).turned(0.0, 0.0, 0.05),
];

/// The frame of the forearm, from inside the elbow into the gauntlet.
const FORE: [Section; 2] = [
    Section::new(0.025, 0.032, 0.032),
    Section::new(-0.07, 0.032, 0.032),
];

/// The gauntlet, from under the elbow to past the wrist: it deepens toward the back of the arm,
/// carries a stripe and ends in a cuff. It stays narrow, so the braced arms fit the collider.
const GAUNTLET: [Section; 5] = [
    Section::new(-0.04, 0.038, 0.045).back(0.004),
    Section::new(-0.085, 0.048, 0.07).back(0.014),
    Section::new(-0.17, 0.05, 0.076).back(0.016),
    Section::new(-0.19, 0.05, 0.076).back(0.016),
    Section::new(-FOREARM - 0.03, 0.044, 0.064).back(0.012),
];

/// The palm, the knuckle plate on its back, the fingers and the thumb pointing forward.
const HAND: [Plate; 4] = [
    Plate::new([0.0, -0.035, 0.0], [0.024, 0.045, 0.034], [DARK, DARK]).cut(0.008),
    Plate::new([0.0, -0.03, 0.035], [0.02, 0.03, 0.007], [SHELL, SHELL])
        .cut(0.0)
        .fine(),
    Plate::new([0.0, -0.088, -0.008], [0.022, 0.016, 0.028], [DARK, DARK]).cut(0.0),
    Plate::new([0.0, -0.04, -0.04], [0.012, 0.022, 0.014], [DARK, DARK])
        .turned(0.3, 0.0, 0.0)
        .cut(0.0)
        .fine(),
];

/// Builds the upper arm of the right side at the level `detail`, with the larger shoulder plate
/// when `long`.
pub fn upper_arm(detail: Detail, long: bool) -> Soup {
    let mut soup = Soup::default();
    loft(
        &mut soup,
        &UPPER,
        detail.ring(Ring::Round(8, 2.0)),
        |_, _| DARK,
    );
    discs(&mut soup, &JOINTS, detail);
    let plate: &[Plate] = if long { &LONG } else { &SHORT };
    plates(&mut soup, plate, detail);
    soup
}

/// Builds a forearm at the level `detail`.
pub fn forearm(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    if detail.near() {
        loft(&mut soup, &FORE, Ring::Round(8, 2.0), |_, _| DARK);
    }
    let paint = |span: usize, _| [SHELL, SHELL, ACCENT, SHADE][span];
    loft(&mut soup, &GAUNTLET, detail.ring(Ring::Cut(LARGE)), paint);
    soup
}

/// Builds a hand at the level `detail`.
pub fn hand(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    plates(&mut soup, &HAND, detail);
    soup
}
