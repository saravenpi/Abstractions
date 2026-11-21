//! The pelvis: a dark core with a ball at the waist, a belt plate, a front and a rear plate that
//! stay above its underside, and a housing for each hip.

use bevy::prelude::*;

use crate::avatar::tune::{DARK, HIP, SHELL, WAIST};
use crate::skin::{Detail, Disc, LARGE, Plate, Ring, Section, Soup, discs, loft, orb, plates};

/// The core from its underside to the waist.
const CORE: [Section; 4] = [
    Section::new(-0.08, 0.085, 0.07),
    Section::new(-0.035, 0.125, 0.095),
    Section::new(0.035, 0.12, 0.09),
    Section::new(0.07, 0.085, 0.068),
];

/// The belt plate, the front plate under it and the rear plate.
const PLATES: [Plate; 3] = [
    Plate::new([0.0, 0.02, -0.092], [0.075, 0.032, 0.012], [SHELL, SHELL]),
    Plate::new([0.0, -0.04, -0.088], [0.05, 0.034, 0.012], [SHELL, SHELL])
        .turned(0.15, 0.0, 0.0)
        .fine(),
    Plate::new([0.0, -0.012, 0.094], [0.1, 0.055, 0.014], [SHELL, SHELL]).turned(-0.12, 0.0, 0.0),
];

/// The housing of each hip.
const DISCS: [Disc; 2] = [
    Disc::new([HIP[0], HIP[1], HIP[2]], Vec3::X, [0.066, 0.052], DARK).fine(),
    Disc::new([-HIP[0], HIP[1], HIP[2]], Vec3::X, [0.066, 0.052], DARK).fine(),
];

/// Builds the pelvis at the level `detail`.
pub fn pelvis(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    loft(&mut soup, &CORE, detail.ring(Ring::Cut(LARGE)), |_, _| DARK);
    if detail.near() {
        let ball = (Ring::Round(6, 2.0), DARK);
        orb(&mut soup, Vec3::from_array(WAIST), 0.062, ball);
    }
    plates(&mut soup, &PLATES, detail);
    discs(&mut soup, &DISCS, detail);
    soup
}
