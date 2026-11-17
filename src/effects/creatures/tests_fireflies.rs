//! Fireflies on a small globe: they stay in their flight band over the ground while the camera
//! walks and the anchor is rebased under it.

use bevy::prelude::*;

use super::env::Ctx;
use super::fireflies::{self, Flight};
use super::tests_globe::{Walk, sky};
use super::wander::{Wander, ground};
use crate::effects::ground::Ground;
use crate::effects::rng::Rng;

/// Flies every firefly of `flies` one frame of `walk` and widens `range` to the lowest and the
/// highest any of them flew over the ground.
fn fly_fireflies(
    walk: &mut Walk,
    frame: usize,
    flies: &mut [(Wander, Transform)],
    ctx: (&mut Rng, &mut (f32, f32)),
) {
    let (rng, range) = ctx;
    let shift = walk.step();
    let eye = walk.eye();
    let flight = Flight {
        sky: sky(&eye, frame),
        level: 24.0,
    };
    let mut ctx = Ctx {
        ground: Ground::new(&walk.anchor, &walk.terrain, &eye),
        rng,
    };
    for (wander, at) in flies.iter_mut() {
        if let Some(shift) = shift {
            at.translation = shift.point(at.translation);
            wander.target = shift.point(wander.target);
        }
        fireflies::fly(wander, at, &flight, eye.at, &mut ctx);
        let over = ctx.ground.altitude(at.translation) - ground(&ctx, at.translation);
        *range = (range.0.min(over), range.1.max(over));
    }
}

/// While the camera walks 500 m over a 3 km globe at night, every firefly stays near the
/// ground: within its flight band over the ground or the sea, give or take its hover.
#[test]
fn fireflies_stay_near_the_ground_on_a_small_globe() {
    let (mut walk, mut rng) = (Walk::new(), Rng::default());
    let mut flies: Vec<(Wander, Transform)> = (0..24)
        .map(|slot| {
            (
                Wander::new(slot),
                Transform::from_translation(super::wander::PARKED),
            )
        })
        .collect();
    let mut range = (f32::MAX, f32::MIN);
    for frame in 0..3100 {
        fly_fireflies(&mut walk, frame, &mut flies, (&mut rng, &mut range));
    }
    assert!(
        walk.walked >= 500.0 && walk.rebases >= 10,
        "{}",
        walk.rebases
    );
    let (band, (low, high)) = (fireflies::ROAM.band, range);
    assert!(
        low > band.0 - 1.5 && high < band.1 + 1.5,
        "{low} to {high} m"
    );
}
