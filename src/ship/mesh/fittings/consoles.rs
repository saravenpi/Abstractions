//! The side consoles: on each side of the seat a dark frame along the wall of the tub, low at
//! the front, where the gear lever and the warp switch are, and higher beside the thighs, where
//! it is the armrest the stick and the throttle hinge on. Each has a light top plate, a row of
//! dark keys on it and a cable run above it along the wall, and on the floor between them lies
//! the heel plate.

use bevy::prelude::*;

use super::beam;
use crate::ship::cockpit::{BACK, FLOOR, STICK};
use crate::ship::solid::Solid;

/// Height of the top of the front of the side consoles, where the gear lever and the warp
/// switch are, and of the armrest behind it, where the stick and the throttle hinge.
pub const CONSOLE: (f32, f32) = (0.25, 0.31);

/// Where a console starts and ends across the tub, on the right, in metres: its inner face and
/// its outer face, which is in the wall.
const ACROSS: (f32, f32) = (0.25, 0.39);

/// Where the front, the step and the back of a console are along z.
const ALONG: (f32, f32, f32) = (-1.84, -1.58, -0.98);

/// Returns the dark frames of both consoles: the low front, the armrest, and on it the boot of
/// the control that hinges there.
pub fn frames() -> Solid {
    let mut right = Solid::default();
    let v = Vec3::new;
    let (near, far) = ACROSS;
    let front = (v(near, FLOOR, ALONG.0), v(far, CONSOLE.0 - 0.012, ALONG.1));
    let back = (v(near, FLOOR, ALONG.1), v(far, CONSOLE.1 - 0.012, ALONG.2));
    beam(&mut right, front, 0.015, 0);
    beam(&mut right, back, 0.015, 0);
    let boot = (
        v(STICK.x - 0.03, STICK.y - 0.01, STICK.z - 0.05),
        v(STICK.x + 0.03, STICK.y + 0.02, STICK.z + 0.05),
    );
    right.slab(boot.0, boot.1);
    right.mirrored()
}

/// Returns the light parts of both consoles: the top plates, bone, with their dark keys, the
/// cable runs along the walls, slate, and the heel plate on the floor, grey.
pub fn tops() -> Solid {
    let mut right = Solid::default();
    let v = Vec3::new;
    let (near, far) = (ACROSS.0 + 0.012, ACROSS.1 - 0.02);
    right.slab(
        v(near, CONSOLE.0 - 0.03, ALONG.0 + 0.012),
        v(far, CONSOLE.0, ALONG.1 - 0.012),
    );
    right.slab(
        v(near, CONSOLE.1 - 0.03, ALONG.1 + 0.012),
        v(far, CONSOLE.1, ALONG.2 - 0.012),
    );
    right.set_tone(3);
    for k in 0..4 {
        let z = -1.12 + 0.035 * k as f32;
        right.slab(
            v(0.275, CONSOLE.1 - 0.01, z),
            v(0.345, CONSOLE.1 + 0.008, z + 0.02),
        );
    }
    right.set_tone(1);
    right.slab(v(0.345, 0.345, -1.78), v(0.39, 0.375, BACK + 0.07));
    let mut all = right.mirrored();
    all.set_tone(2);
    all.slab(v(-0.2, FLOOR - 0.01, -1.98), v(0.2, FLOOR + 0.012, -1.5));
    all
}
