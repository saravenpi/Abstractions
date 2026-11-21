//! The torso: a slim dark waist, a chest shell with its proud plate and marking, the collar, a
//! housing for each shoulder, and the pack on the back that carries the wing roots.

use bevy::prelude::*;

use crate::avatar::tune::{ACCENT, DARK, SHADE, SHELL, SHOULDER};
use crate::skin::{Detail, Disc, LARGE, Plate, Ring, Section, Soup, discs, loft, plates};

/// The waist frame from under the waist joint up into the chest.
const WAIST: [Section; 3] = [
    Section::new(-0.03, 0.08, 0.065),
    Section::new(0.08, 0.085, 0.07),
    Section::new(0.16, 0.1, 0.075),
];

/// The chest shell from the ribs to the yoke, its front further out than its back.
const RIBS: [Section; 5] = [
    Section::new(0.12, 0.105, 0.08).back(-0.008),
    Section::new(0.2, 0.165, 0.105).back(-0.01),
    Section::new(0.32, 0.185, 0.11).back(-0.008),
    Section::new(0.375, 0.165, 0.1).back(-0.006),
    Section::new(0.4, 0.11, 0.07),
];

/// The collar round the neck housing, wider at its top, closed by a dark lid.
const COLLAR: [Section; 3] = [
    Section::new(0.385, 0.105, 0.085),
    Section::new(0.445, 0.12, 0.1),
    Section::new(0.45, 0.1, 0.082),
];

/// The two breast plates that meet in a V, two belly lames, the back plate with its slate
/// inset and two vents, and the wing roots on its sides. The back plate and what it carries stay
/// in front of the backrest of the seat, which the seated torso leans on.
const PLATES: [Plate; 10] = [
    Plate::new([0.062, 0.29, -0.113], [0.066, 0.058, 0.012], [SHELL, SHELL]).turned(0.0, -0.1, 0.2),
    Plate::new(
        [-0.062, 0.29, -0.113],
        [0.066, 0.058, 0.012],
        [SHELL, SHELL],
    )
    .turned(0.0, 0.1, -0.2),
    Plate::new([0.0, 0.045, -0.068], [0.055, 0.03, 0.01], [SHELL, SHELL])
        .cut(0.0)
        .fine(),
    Plate::new([0.0, 0.105, -0.074], [0.065, 0.03, 0.01], [SHELL, SHELL])
        .cut(0.0)
        .fine(),
    Plate::new([0.0, 0.225, 0.094], [0.128, 0.105, 0.03], [SHELL, SHELL]).cut(LARGE),
    Plate::new([0.0, 0.215, 0.119], [0.085, 0.06, 0.008], [SHADE, SHADE])
        .cut(0.0)
        .fine(),
    Plate::new([0.05, 0.155, 0.123], [0.026, 0.016, 0.008], [DARK, DARK])
        .cut(0.0)
        .fine(),
    Plate::new([-0.05, 0.155, 0.123], [0.026, 0.016, 0.008], [DARK, DARK])
        .cut(0.0)
        .fine(),
    Plate::new([0.138, 0.225, 0.073], [0.02, 0.095, 0.03], [DARK, DARK])
        .cut(0.0)
        .fine(),
    Plate::new([-0.138, 0.225, 0.073], [0.02, 0.095, 0.03], [DARK, DARK])
        .cut(0.0)
        .fine(),
];

/// How far a shoulder housing reaches into the chest from its joint, in meters.
const SUNK: f32 = 0.015;

/// The housing of each shoulder, sunk into the chest, the roll on top of the pack with its two
/// accent straps, and the triangle marking on the left breast.
const DISCS: [Disc; 6] = [
    Disc::new(
        [SHOULDER[0] - SUNK, SHOULDER[1], 0.0],
        Vec3::X,
        [0.056, 0.045],
        DARK,
    )
    .fine(),
    Disc::new(
        [SUNK - SHOULDER[0], SHOULDER[1], 0.0],
        Vec3::X,
        [0.056, 0.045],
        DARK,
    )
    .fine(),
    Disc::new([0.0, 0.317, 0.1], Vec3::X, [0.04, 0.115], SHADE).fine(),
    Disc::new([0.07, 0.317, 0.1], Vec3::X, [0.044, 0.012], ACCENT).fine(),
    Disc::new([-0.07, 0.317, 0.1], Vec3::X, [0.044, 0.012], ACCENT).fine(),
    Disc::new([-0.07, 0.3, -0.127], Vec3::Z, [0.026, 0.005], ACCENT)
        .sides(3)
        .fine(),
];

/// Builds the torso at the level `detail`.
pub fn torso(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    loft(
        &mut soup,
        &WAIST,
        detail.ring(Ring::Round(8, 2.4)),
        |_, _| DARK,
    );
    let under = |span: usize, _| if span == 0 { SHADE } else { SHELL };
    loft(&mut soup, &RIBS, detail.ring(Ring::Cut(LARGE)), under);
    if detail.near() {
        let lid = |span: usize, _| if span == 1 { DARK } else { SHADE };
        loft(&mut soup, &COLLAR, Ring::Round(10, 2.2), lid);
    }
    plates(&mut soup, &PLATES, detail);
    discs(&mut soup, &DISCS, detail);
    soup
}
