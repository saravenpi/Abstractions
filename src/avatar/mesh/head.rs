//! The head: a big round helmet on a thin neck, the dark visor box that holds the screen, an ear
//! disc on each side, and a side vane growing out of each ear: a flat horizontal blade swept
//! back. The vanes are the antenna and the mark of the silhouette; nothing stands over the visor.

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

/// Height of the side vanes above the neck joint: the height of the ears, in meters.
const EAR: f32 = MID - 0.01;

/// How far a side vane is swept back from straight out to the side, in radians.
const SWEEP: f32 = 1.0;

/// Half extents of a side vane, along its length, its thickness and its chord, and of its accent
/// tip. A vane starts in the back of its ear housing and ends 6 cm outside the ear, behind an arm
/// raised overhead, in front of the headrest of the seat and inside the player collider in a
/// banked run.
const VANE: [[f32; 3]; 2] = [[0.06, 0.016, 0.03], [0.014, 0.02, 0.034]];

/// The visor box, the two side vanes with their accent tips, and the slate hatch with its two
/// dark vents on the back of the helmet.
const PLATES: [Plate; 8] = [
    Plate::new(
        [0.0, MID, 0.002 - SCREEN_FRONT + VISOR[2]],
        VISOR,
        [DARK, DARK],
    )
    .cut(LARGE),
    Plate::new([0.204, EAR, 0.105], VANE[0], [SHELL, SHELL]).turned(0.0, -SWEEP, 0.0),
    Plate::new([-0.204, EAR, 0.105], VANE[0], [SHELL, SHELL]).turned(0.0, SWEEP, 0.0),
    Plate::new([0.23, EAR, 0.146], VANE[1], [ACCENT, ACCENT])
        .turned(0.0, -SWEEP, 0.0)
        .cut(0.0),
    Plate::new([-0.23, EAR, 0.146], VANE[1], [ACCENT, ACCENT])
        .turned(0.0, SWEEP, 0.0)
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
    Disc::new([0.178, EAR, 0.015], Vec3::X, [0.07, 0.022], DARK),
    Disc::new([-0.178, EAR, 0.015], Vec3::X, [0.07, 0.022], DARK),
    Disc::new([0.202, EAR, 0.015], Vec3::X, [0.04, 0.008], SHELL)
        .sides(8)
        .fine(),
    Disc::new([-0.202, EAR, 0.015], Vec3::X, [0.04, 0.008], SHELL)
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
