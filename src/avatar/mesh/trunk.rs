//! The trunk segments: pelvis, torso and the head with its screen bezel, ears and antenna.

use bevy::prelude::*;

use super::shapes::{Tube, ball, rounded, tube};
use crate::skin::Soup;
use crate::avatar::tune::{
    ACCENT, CAP_RADIUS, CHEST, DARK, HEAD, HIP, HIPS, NECK, SCREEN, SHADE, SHELL, SHOULDER,
};

/// Distance of the lit screen in front of the middle of the head, in meters.
pub const SCREEN_FRONT: f32 = HEAD.1[2] + 0.012;

/// Returns the shell colour above the normalised height `line` and the shaded one below.
fn banded(line: f32) -> impl Fn(Vec3) -> [f32; 3] {
    move |at: Vec3| if at.y < line { SHADE } else { SHELL }
}

/// Builds the pelvis: a shaded block with a dark cap at each hip.
pub fn pelvis() -> Soup {
    let mut soup = Soup::default();
    let block = (Vec3::new(0.0, -0.005, 0.0), Vec3::from_array(HIPS));
    rounded(&mut soup, block, 2, |_| SHADE);
    for sign in [-1.0, 1.0] {
        ball(
            &mut soup,
            [HIP[0] * sign, HIP[1], HIP[2]],
            CAP_RADIUS[3],
            DARK,
        );
    }
    soup
}

/// Builds the torso: a compact shell, an accent bar on the chest, a neck cap and a dark cap at each shoulder.
pub fn torso() -> Soup {
    let mut soup = Soup::default();
    let block = (Vec3::new(0.0, CHEST.0, 0.0), Vec3::from_array(CHEST.1));
    rounded(&mut soup, block, 2, banded(-0.4));
    let front = -CHEST.1[2];
    let bar = Tube::drum(
        [-0.055, 0.27, front + 0.01],
        [0.055, 0.27, front + 0.01],
        0.022,
        4,
    );
    tube(&mut soup, &bar, ACCENT);
    ball(&mut soup, NECK, 0.07, DARK);
    for sign in [-1.0, 1.0] {
        ball(
            &mut soup,
            [SHOULDER[0] * sign, SHOULDER[1], SHOULDER[2]],
            CAP_RADIUS[0],
            DARK,
        );
    }
    soup
}

/// Adds the dark bezel the screen sits in: a plate just proud of the face, with a skirt back into the head.
fn bezel(soup: &mut Soup) {
    let (w, h) = (SCREEN[0] + 0.014, SCREEN[1] + 0.014);
    let (front, back) = (-(SCREEN_FRONT - 0.002), -HEAD.1[2] * 0.7);
    let ring = |z: f32, grow: f32| {
        [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .map(|(x, y): (f32, f32)| Vec3::new(x * (w + grow), HEAD.0 + y * (h + grow), z))
    };
    let (near, far) = (ring(front, 0.0), ring(back, 0.01));
    soup.quad(near, Vec3::NEG_Z, DARK);
    for side in 0..4 {
        let next = (side + 1) % 4;
        let out = (near[side] + near[next]) * 0.5 - Vec3::new(0.0, HEAD.0, front);
        soup.quad([near[side], near[next], far[next], far[side]], out, DARK);
    }
}

/// Builds the head: a rounded shell on a short neck, the bezel, an accent ear on each side and an antenna.
pub fn head() -> Soup {
    let mut soup = Soup::default();
    let (half, top) = (Vec3::from_array(HEAD.1), HEAD.0 + HEAD.1[1]);
    rounded(&mut soup, (Vec3::Y * HEAD.0, half), 3, banded(-0.55));
    bezel(&mut soup);
    tube(
        &mut soup,
        &Tube::open([0.0, -0.03, 0.0], [0.0, 0.07, 0.0], (0.05, 0.05)),
        DARK,
    );
    for sign in [-1.0, 1.0] {
        let (inner, outer) = ((half.x - 0.012) * sign, (half.x + 0.016) * sign);
        let ear = Tube::drum([inner, HEAD.0, 0.0], [outer, HEAD.0, 0.0], 0.045, 6);
        tube(&mut soup, &ear, ACCENT);
    }
    let stem = Tube {
        sides: 3,
        ..Tube::open(
            [0.0, top - 0.01, 0.02],
            [0.0, top + 0.04, 0.03],
            (0.008, 0.006),
        )
    };
    tube(&mut soup, &stem, DARK);
    ball(&mut soup, [0.0, top + 0.045, 0.03], 0.014, ACCENT);
    soup
}
