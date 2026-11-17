//! The limb segments, each hanging along -Y from its joint: arms, hands, legs and the boots.

use bevy::prelude::*;

use super::shapes::{Tube, ball, rounded, tube};
use super::soup::Soup;
use crate::avatar::tune::{
    ACCENT, ANKLE_RISE, BOOT, CAP_RADIUS, DARK, FOREARM, LIMB_RADIUS, NOZZLE, SHADE, SHELL, SHIN,
    THIGH, UPPER_ARM,
};

/// Builds an upper arm: a slim tube ending in the dark drum of the elbow.
pub fn upper_arm() -> Soup {
    let mut soup = Soup::default();
    let r = LIMB_RADIUS[0];
    tube(
        &mut soup,
        &Tube::open([0.0, 0.0, 0.0], [0.0, -UPPER_ARM, 0.0], (r, r * 0.92)),
        SHELL,
    );
    let elbow = Tube::drum(
        [-0.042, -UPPER_ARM, 0.0],
        [0.042, -UPPER_ARM, 0.0],
        CAP_RADIUS[1],
        5,
    );
    tube(&mut soup, &elbow, DARK);
    soup
}

/// Builds a forearm: a tube that widens toward an accent cuff, and the dark cap of the wrist.
pub fn forearm() -> Soup {
    let mut soup = Soup::default();
    let r = LIMB_RADIUS[1];
    tube(
        &mut soup,
        &Tube::open([0.0, 0.0, 0.0], [0.0, -FOREARM, 0.0], (r, r * 1.12)),
        SHELL,
    );
    let cuff = Tube::drum(
        [0.0, -FOREARM + 0.065, 0.0],
        [0.0, -FOREARM + 0.02, 0.0],
        r * 1.3,
        5,
    );
    tube(&mut soup, &cuff, ACCENT);
    ball(&mut soup, [0.0, -FOREARM, 0.0], CAP_RADIUS[2], DARK);
    soup
}

/// Builds a hand: a mitten with a thumb pointing forward. Both sides share it.
pub fn hand() -> Soup {
    let mut soup = Soup::default();
    let mitten = (Vec3::new(0.0, -0.045, 0.0), Vec3::new(0.026, 0.048, 0.038));
    let thumb = (
        Vec3::new(0.0, -0.03, -0.042),
        Vec3::new(0.014, 0.022, 0.018),
    );
    rounded(&mut soup, mitten, 1, |_| SHADE);
    rounded(&mut soup, thumb, 1, |_| SHADE);
    soup
}

/// Builds a thigh: a tube ending in the dark drum of the knee, which stays visible.
pub fn thigh() -> Soup {
    let mut soup = Soup::default();
    let r = LIMB_RADIUS[2];
    tube(
        &mut soup,
        &Tube::open([0.0, 0.0, 0.0], [0.0, -THIGH, 0.0], (r, r * 0.86)),
        SHELL,
    );
    let knee = Tube::drum(
        [-0.056, -THIGH, 0.0],
        [0.056, -THIGH, 0.0],
        CAP_RADIUS[4],
        6,
    );
    tube(&mut soup, &knee, DARK);
    soup
}

/// Builds a shin: a tube that flares toward the boot, an accent band and the dark cap of the ankle.
pub fn shin() -> Soup {
    let mut soup = Soup::default();
    let r = LIMB_RADIUS[3];
    tube(
        &mut soup,
        &Tube::open([0.0, 0.0, 0.0], [0.0, -SHIN, 0.0], (r * 0.9, r * 1.1)),
        SHELL,
    );
    let band = Tube::drum(
        [0.0, -SHIN + 0.1, 0.0],
        [0.0, -SHIN + 0.055, 0.0],
        r * 1.22,
        5,
    );
    tube(&mut soup, &band, ACCENT);
    ball(&mut soup, [0.0, -SHIN, 0.0], CAP_RADIUS[5], DARK);
    soup
}

/// Builds a boot: a slightly oversized rounded shoe with a dark sole and the booster nozzle under it.
pub fn boot() -> Soup {
    let mut soup = Soup::default();
    let shoe = (Vec3::from_array(BOOT.0), Vec3::from_array(BOOT.1));
    rounded(
        &mut soup,
        shoe,
        2,
        |at| if at.y < -0.45 { DARK } else { SHADE },
    );
    let nozzle = Tube::open(
        [0.0, -ANKLE_RISE + 0.03, NOZZLE.1],
        [0.0, -ANKLE_RISE, NOZZLE.1],
        (NOZZLE.0 * 0.85, NOZZLE.0),
    );
    tube(&mut soup, &nozzle, DARK);
    soup
}
