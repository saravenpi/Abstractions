//! The head: a big round helmet on a thin neck, the dark visor box that holds the screen, an ear
//! disc on each side, and the brow wing: two blades over the visor that reach past
//! the helmet in a shallow V. The wing is the antenna and the mark of the silhouette.

use bevy::prelude::*;

use crate::avatar::tune::{ACCENT, DARK, HEAD, SCREEN, SHADE, SHELL};
use crate::skin::{Detail, Disc, LARGE, Plate, Ring, Section, Soup, discs, loft, plates};

/// Distance of the lit screen in front of the middle of the head, in meters.
pub const SCREEN_FRONT: f32 = HEAD.1[2] + 0.012;

/// Height of the middle of the head above the neck joint, in meters.
const MID: f32 = HEAD.0;

/// Half extents of the visor box, in meters: the screen and a dark margin around it.
const VISOR: [f32; 3] = [SCREEN[0] + 0.024, SCREEN[1] + 0.024, 0.048];

/// The helmet from its rim to its crown: height above the neck joint, half width, half depth.
const DOME: [Section; 6] = [
    Section::new(MID - HEAD.1[1], 0.115, 0.105),
    Section::new(0.075, 0.168, 0.145),
    Section::new(MID, HEAD.1[0], HEAD.1[2]),
    Section::new(0.285, 0.165, 0.145),
    Section::new(0.33, 0.125, 0.115),
    Section::new(MID + HEAD.1[1], 0.06, 0.06),
];

/// Half extents of a blade of the brow wing, and how far it rolls up and sweeps back, in radians.
/// The wing is as wide as a run banked into a turn leaves room for inside the player collider.
const BLADE: ([f32; 3], f32, f32) = ([0.108, 0.015, 0.042], 0.24, 0.2);

/// The visor box, the two blades of the brow wing with their accent tips, and the slate hatch
/// with its two dark vents on the back of the helmet.
const PLATES: [Plate; 8] = [
    Plate::new(
        [0.0, MID, 0.002 - SCREEN_FRONT + VISOR[2]],
        VISOR,
        [DARK, DARK],
    )
    .cut(LARGE),
    Plate::new([0.121, MID + 0.11, -0.119], BLADE.0, [SHELL, SHELL]).turned(0.1, -BLADE.2, BLADE.1),
    Plate::new([-0.121, MID + 0.11, -0.119], BLADE.0, [SHELL, SHELL])
        .turned(0.1, BLADE.2, -BLADE.1),
    Plate::new(
        [0.212, MID + 0.133, -0.101],
        [0.02, 0.019, 0.046],
        [ACCENT, ACCENT],
    )
    .turned(0.1, -BLADE.2, BLADE.1)
    .cut(0.0),
    Plate::new(
        [-0.212, MID + 0.133, -0.101],
        [0.02, 0.019, 0.046],
        [ACCENT, ACCENT],
    )
    .turned(0.1, BLADE.2, -BLADE.1)
    .cut(0.0),
    Plate::new(
        [0.0, MID + 0.015, 0.143],
        [0.075, 0.055, 0.01],
        [SHADE, SHADE],
    )
    .cut(0.0)
    .fine(),
    Plate::new([0.0, MID + 0.035, 0.15], [0.05, 0.008, 0.005], [DARK, DARK])
        .cut(0.0)
        .fine(),
    Plate::new([0.0, MID + 0.005, 0.15], [0.05, 0.008, 0.005], [DARK, DARK])
        .cut(0.0)
        .fine(),
];

/// The neck, and on each side the dark ear disc and its bone cap.
const DISCS: [Disc; 5] = [
    Disc::new([0.0, 0.02, 0.0], Vec3::Y, [0.042, 0.05], DARK)
        .sides(8)
        .fine(),
    Disc::new([0.178, MID - 0.01, 0.015], Vec3::X, [0.07, 0.022], DARK),
    Disc::new([-0.178, MID - 0.01, 0.015], Vec3::X, [0.07, 0.022], DARK),
    Disc::new([0.202, MID - 0.01, 0.015], Vec3::X, [0.04, 0.008], SHELL)
        .sides(8)
        .fine(),
    Disc::new([-0.202, MID - 0.01, 0.015], Vec3::X, [0.04, 0.008], SHELL)
        .sides(8)
        .fine(),
];

/// Builds the head at the level `detail`.
pub fn head(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    let rim = |span: usize, _| if span == 0 { SHADE } else { SHELL };
    loft(&mut soup, &DOME, detail.ring(Ring::Round(12, 2.6)), rim);
    plates(&mut soup, &PLATES, detail);
    discs(&mut soup, &DISCS, detail);
    soup
}
