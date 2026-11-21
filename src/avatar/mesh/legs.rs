//! The legs, each hanging along -Y from its joint, written for the right side: a thin dark thigh
//! under a tasset hinged at the hip disc, a greave that widens to the ankle under a knee cap,
//! and the boot. The left side is the mirror.

use bevy::prelude::*;

use crate::avatar::tune::{ACCENT, ANKLE_RISE, BOOT, DARK, SHADE, SHELL, SHIN, THIGH};
use crate::skin::{Detail, Disc, LARGE, Plate, Ring, Section, Soup, discs, loft, plates};

/// The frame of the thigh, from inside the hip to the knee.
const UPPER: [Section; 2] = [
    Section::new(0.03, 0.046, 0.046),
    Section::new(-THIGH, 0.04, 0.04),
];

/// The housing of the knee, the hinge of the tassets at the hip, and the hip disc with its
/// bone cap on the outer tasset.
const JOINTS: [Disc; 4] = [
    Disc::new([0.0, -THIGH, 0.0], Vec3::X, [0.058, 0.052], DARK).fine(),
    Disc::new([0.04, 0.0, 0.0], Vec3::X, [0.05, 0.03], DARK)
        .sides(8)
        .fine(),
    Disc::new([0.082, -0.04, 0.0], Vec3::X, [0.046, 0.008], DARK).fine(),
    Disc::new([0.09, -0.04, 0.0], Vec3::X, [0.026, 0.004], SHELL)
        .sides(8)
        .fine(),
];

/// The tassets hinged at the hip disc: one over the front of the thigh, one on its outside and a
/// short one behind it, all flared, and the stripe near the lower edge of the outer one.
const TASSET: [Plate; 4] = [
    Plate::new([0.005, -0.09, 0.06], [0.065, 0.085, 0.009], [SHADE, SHADE])
        .turned(-0.1, 0.0, 0.0)
        .cut(0.0)
        .fine(),
    Plate::new(
        [0.005, -0.135, -0.06],
        [0.065, 0.125, 0.012],
        [SHADE, SHADE],
    )
    .turned(0.1, 0.0, 0.0),
    Plate::new([0.069, -0.17, 0.0], [0.012, 0.15, 0.075], [SHADE, SHADE]).turned(0.0, 0.0, 0.08),
    Plate::new([0.09, -0.27, 0.0], [0.004, 0.014, 0.066], [ACCENT, ACCENT])
        .turned(0.0, 0.0, 0.08)
        .cut(0.0)
        .fine(),
];

/// The frame of the shin, from inside the knee into the greave.
const LOWER: [Section; 2] = [
    Section::new(0.03, 0.04, 0.04),
    Section::new(-0.08, 0.04, 0.04),
];

/// The greave, from under the knee to the ankle: it widens, bulges at the calf, carries a stripe
/// and ends in a cuff.
const GREAVE: [Section; 5] = [
    Section::new(-0.04, 0.05, 0.055).back(0.004),
    Section::new(-0.13, 0.07, 0.085).back(0.015),
    Section::new(-0.28, 0.08, 0.095).back(0.012),
    Section::new(-0.305, 0.08, 0.095).back(0.012),
    Section::new(-0.35, 0.07, 0.085).back(0.01),
];

/// The knee cap on the front of the greave. It sits under the knee, where the shin of a running
/// leg stays inside the player collider.
const KNEE: [Plate; 1] =
    [Plate::new([0.0, -0.065, -0.056], [0.045, 0.045, 0.016], [SHELL, SHELL]).fine()];

/// The housing of the ankle.
const ANKLE: [Disc; 1] = [Disc::new([0.0, -SHIN, 0.0], Vec3::X, [0.05, 0.076], DARK)
    .sides(8)
    .fine()];

/// Height of the middle of the sole under the ankle, and half its thickness, in meters.
const SOLE: [f32; 2] = [0.018 - ANKLE_RISE, 0.018];

/// The sole, the toe cap, the upper round the ankle and the heel block over the nozzle.
const SHOE: [Plate; 4] = [
    Plate::new(
        [0.0, SOLE[0], BOOT.0[2]],
        [BOOT.1[0], SOLE[1], BOOT.1[2]],
        [DARK, DARK],
    )
    .cut(0.01),
    Plate::new([0.0, -0.048, -0.098], [0.08, 0.036, 0.064], [SHELL, SHELL]).cut(LARGE),
    Plate::new([0.0, -0.03, 0.008], [0.072, 0.05, 0.066], [SHADE, SHADE]).cut(LARGE),
    Plate::new([0.0, -0.05, 0.068], [0.064, 0.036, 0.027], [DARK, DARK])
        .cut(0.0)
        .fine(),
];

/// Builds the thigh of the right side at the level `detail`.
pub fn thigh(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    loft(
        &mut soup,
        &UPPER,
        detail.ring(Ring::Round(8, 2.0)),
        |_, _| DARK,
    );
    discs(&mut soup, &JOINTS, detail);
    plates(&mut soup, &TASSET, detail);
    soup
}

/// Builds a shin at the level `detail`.
pub fn shin(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    if detail.near() {
        loft(&mut soup, &LOWER, Ring::Round(8, 2.0), |_, _| DARK);
    }
    let paint = |span: usize, _| match span {
        2 => ACCENT,
        3 => SHADE,
        _ => SHELL,
    };
    loft(&mut soup, &GREAVE, detail.ring(Ring::Cut(LARGE)), paint);
    plates(&mut soup, &KNEE, detail);
    discs(&mut soup, &ANKLE, detail);
    soup
}

/// Builds a boot at the level `detail`.
pub fn boot(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    plates(&mut soup, &SHOE, detail);
    soup
}
