//! Animals are solid: the body walks into a drawn animal and stops against it, where with no animal it
//! walks on.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::climbing::{FLOOR, stage, walk};
use crate::fauna::{bulk, capsule};
use crate::player::BODY_RADIUS;

/// Length and hip height in metres of the test animal: a 1.8 m grazer with hips 1 m high.
const ANIMAL: (f32, f32) = (1.8, 1.0);

/// Metres ahead of the body at which the test animal stands across its way.
const AHEAD: f32 = 3.0;

/// Returns how far ahead the body is after three seconds walking straight on, with the animal across
/// its way or not.
fn walked(animal: bool) -> f32 {
    let mut app = stage(30.0);
    if animal {
        let across = Transform::from_xyz(0.0, FLOOR + ANIMAL.1, -AHEAD)
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2));
        app.world_mut().spawn((bulk(ANIMAL.0, ANIMAL.1), across));
    }
    let seen = walk(&mut app, 1.0, 90);
    let (transform, _, _) = seen.last().copied().expect("frames");
    -transform.translation.z
}

/// The body walks past where the animal would stand when there is none, and stops against its body
/// when it stands there: never closer to its middle than the two bodies' radii allow.
#[test]
fn the_body_cannot_walk_through_an_animal() {
    assert!(walked(false) > AHEAD + 1.0, "the body did not walk on");
    let ahead = walked(true);
    let (radius, _) = capsule(ANIMAL.0, ANIMAL.1);
    let stop = AHEAD - radius - BODY_RADIUS + 0.05;
    assert!(
        ahead < stop,
        "the body walked {ahead} m, into the animal at {AHEAD} m"
    );
}
