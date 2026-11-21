//! The companion's meshes, built in code. It is the helmet of the robot flying on its own: the
//! same dome, visor box and ear discs, a low crest that rises out of the crown and carries the
//! antenna tip, a nozzle bell, and three lames round the lower back; and its hand, a small
//! gauntlet.

use bevy::prelude::*;

use super::tune::HALF;
use crate::avatar::tune::{ACCENT, DARK, SHADE, SHELL};
use crate::skin::{Detail, Disc, LARGE, Plate, Ring, Section, Soup, discs, loft, plates};

/// Half width and half height of the lit screen in meters. The picture is twice as wide as tall.
pub const SCREEN: Vec2 = Vec2::new(0.118, 0.059);

/// Distance of the lit screen in front of the body centre, in meters.
pub const SCREEN_FRONT: f32 = HALF[2] + 0.012;

/// Where the antenna ends: the top of the crest, above and behind the crown.
pub const ANTENNA_TIP: Vec3 = Vec3::new(0.0, 0.22, 0.108);

/// Where the thruster glows: the mouth of the nozzle bell.
pub const NOZZLE: Vec3 = Vec3::new(0.0, -HALF[1] - 0.03, 0.0);

/// Half extents of the visor box, in meters: the screen and a dark margin around it.
const VISOR: [f32; 3] = [SCREEN.x + 0.021, SCREEN.y + 0.022, 0.044];

/// The dome from its underside to its crown: height, half width, half depth.
const DOME: [Section; 6] = [
    Section::new(-HALF[1], 0.075, 0.065),
    Section::new(-0.115, 0.145, 0.124),
    Section::new(-0.02, HALF[0], HALF[2]),
    Section::new(0.075, 0.164, 0.14),
    Section::new(0.135, 0.118, 0.105),
    Section::new(HALF[1], 0.05, 0.05),
];

/// The nozzle bell under the dome, wider at its mouth.
const BELL: [Section; 2] = [
    Section::new(-HALF[1] - 0.045, 0.085, 0.085),
    Section::new(-HALF[1] + 0.01, 0.055, 0.055),
];

/// The crest: a fairing that starts inside the crown, as wide as the crown is there, and tapers
/// up and back to the antenna tip. It is half as long as a fin would be and has no root to show.
const CREST: [Section; 3] = [
    Section::new(0.115, 0.042, 0.085).back(0.035),
    Section::new(0.185, 0.022, 0.06).back(0.075),
    Section::new(0.215, 0.008, 0.02).back(0.108),
];

/// The visor box, the brow hood, and the three lames round the lower back.
const PLATES: [Plate; 5] = [
    Plate::new(
        [0.0, 0.0, 0.002 - SCREEN_FRONT + VISOR[2]],
        VISOR,
        [DARK, DARK],
    )
    .cut(LARGE),
    Plate::new([0.0, 0.092, -0.128], [0.125, 0.012, 0.036], [SHELL, SHELL])
        .turned(0.14, 0.0, 0.0)
        .fine(),
    Plate::new([0.0, -0.085, 0.134], [0.07, 0.045, 0.008], [SHADE, SHADE])
        .turned(-0.3, 0.0, 0.0)
        .fine(),
    Plate::new(
        [0.118, -0.085, 0.095],
        [0.055, 0.045, 0.008],
        [SHADE, SHADE],
    )
    .turned(-0.3, 0.85, 0.0)
    .fine(),
    Plate::new(
        [-0.118, -0.085, 0.095],
        [0.055, 0.045, 0.008],
        [SHADE, SHADE],
    )
    .turned(-0.3, -0.85, 0.0)
    .fine(),
];

/// On each side the dark ear disc and its accent centre.
const DISCS: [Disc; 4] = [
    Disc::new([0.168, 0.0, 0.012], Vec3::X, [0.052, 0.02], DARK),
    Disc::new([-0.168, 0.0, 0.012], Vec3::X, [0.052, 0.02], DARK),
    Disc::new([0.188, 0.0, 0.012], Vec3::X, [0.03, 0.007], ACCENT).sides(8),
    Disc::new([-0.188, 0.0, 0.012], Vec3::X, [0.03, 0.007], ACCENT).sides(8),
];

/// The hand from its fingers to its cuff: a small gauntlet with the stripe of the robot's.
const MITT: [Section; 5] = [
    Section::new(-0.055, 0.024, 0.032),
    Section::new(-0.02, 0.03, 0.04),
    Section::new(0.03, 0.03, 0.04),
    Section::new(0.045, 0.03, 0.04),
    Section::new(0.06, 0.022, 0.03),
];

/// Builds the head-body at the level `detail`.
pub fn shell(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    let belly = |span: usize, _| if span == 0 { SHADE } else { SHELL };
    loft(&mut soup, &DOME, detail.ring(Ring::Round(12, 2.6)), belly);
    loft(
        &mut soup,
        &BELL,
        detail.ring(Ring::Round(10, 2.0)),
        |_, _| DARK,
    );
    loft(
        &mut soup,
        &CREST,
        detail.ring(Ring::Round(8, 2.0)),
        |_, _| SHELL,
    );
    plates(&mut soup, &PLATES, detail);
    discs(&mut soup, &DISCS, detail);
    soup
}

/// Builds a hand at the level `detail`: the gauntlet up close, a plain box from afar.
pub fn mitt(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    if detail.near() {
        let paint = |span: usize, _| [SHELL, SHELL, ACCENT, DARK][span];
        loft(&mut soup, &MITT, Ring::Cut(0.012), paint);
    } else {
        let size = [MITT[1].half.x, 0.0575, MITT[1].half.y];
        let plain = Plate::new([0.0, 0.0025, 0.0], size, [SHELL, SHELL]);
        plates(&mut soup, &[plain], detail);
    }
    soup
}
