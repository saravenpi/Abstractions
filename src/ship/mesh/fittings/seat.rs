//! The seat: a dark shell with side wings and a headrest on a foot, three stacked back pads like
//! the lames of the hull, a pan pad with thigh bolsters, and a harness.

use bevy::prelude::*;

use super::beam;
use crate::ship::cockpit::{BACK, FLOOR, PAN};
use crate::ship::solid::Solid;

/// Where the back of the shell is along z: the rear wall of the tub.
const REAR: f32 = -0.9;

/// Half width of the shell and of the pads, in metres.
const HALF: (f32, f32) = (0.25, 0.2);

/// The back pads from the lowest up: the heights each starts and ends at. Each overlaps the one
/// under it and stands a little further back.
const PADS: [(f32, f32); 3] = [(0.1, 0.3), (0.27, 0.48), (0.45, 0.68)];

/// Thickness of a strap of the harness, in metres: it lies on the face of the pads.
const STRAP: f32 = 0.008;

/// Returns the dark shell of the seat: the foot on the floor, the pan with its thigh bolsters,
/// the back with its side wings, and the headrest.
pub fn shell() -> Solid {
    let v = Vec3::new;
    let middle = [
        (v(-0.15, FLOOR, -1.32), v(0.15, -0.02, -1.0)),
        (v(-HALF.0, -0.03, -1.44), v(HALF.0, 0.02, REAR)),
        (v(-HALF.0, 0.0, BACK + 0.04), v(HALF.0, 0.7, REAR)),
        (v(-0.13, 0.68, BACK + 0.03), v(0.13, 1.02, REAR)),
    ];
    let sides = [
        (v(HALF.1, 0.0, -1.42), v(HALF.0, 0.13, BACK)),
        (v(HALF.1, 0.1, BACK - 0.09), v(HALF.0, 0.62, BACK + 0.04)),
    ];
    let (mut shell, mut wings) = (Solid::default(), Solid::default());
    for ends in middle {
        beam(&mut shell, ends, 0.015, 0);
    }
    for ends in sides {
        beam(&mut wings, ends, 0.015, 0);
    }
    shell.merge(&wings.mirrored(), Vec3::ZERO);
    shell
}

/// Returns the pads of the seat, in the slate: the pan, the three back pads and the headrest.
pub fn pads() -> Solid {
    let mut pads = Solid::default();
    let v = Vec3::new;
    beam(
        &mut pads,
        (v(-HALF.1, 0.01, -1.42), v(HALF.1, PAN, BACK)),
        0.006,
        1,
    );
    for (k, (from, to)) in PADS.into_iter().enumerate() {
        let back = BACK + 0.012 * (2 - k) as f32;
        beam(
            &mut pads,
            (v(-HALF.1, from, back), v(HALF.1, to, BACK + 0.05)),
            0.006,
            1,
        );
    }
    beam(
        &mut pads,
        (v(-0.1, 0.74, BACK), v(0.1, 0.98, BACK + 0.04)),
        0.006,
        1,
    );
    pads
}

/// Returns the harness: the two shoulder straps down the back pads, the lap belt across the pan
/// and its dark buckle.
pub fn harness() -> Solid {
    let mut harness = Solid::default();
    let v = Vec3::new;
    for middle in [-0.08, 0.08] {
        let (lo, hi) = (
            v(middle - 0.02, 0.14, BACK - STRAP),
            v(middle + 0.02, 0.7, BACK + 0.01),
        );
        harness.slab(lo, hi);
    }
    harness.slab(v(-HALF.1, PAN - 0.01, -1.12), v(HALF.1, PAN + STRAP, -1.07));
    harness.set_tone(3);
    harness.slab(v(-0.035, PAN, -1.135), v(0.035, PAN + 0.02, -1.055));
    harness
}
