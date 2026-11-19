//! Step-up: the body walks onto low steps and up a stair, a tall ledge stops it, it walks down
//! a stair on the ground, and the camera never snaps up with a step.

use bevy::prelude::*;

use super::climbing::{FLOOR, RISE, RUN, SOLES, block, put, soles, stage, stair, walk};
use crate::player::BODY_RADIUS;

/// A 0.29 m box is climbed at a walk, the body stays on the ground on top of it, and the camera
/// rises with it smoothly: never a large part of the step in one frame, and back on the eyes
/// soon after.
#[test]
fn a_low_box_is_climbed() {
    let mut app = stage(30.0);
    block(&mut app, (1.5, 30.0), RISE);
    let seen = walk(&mut app, 1.0, 45);
    let (transform, motion, _) = seen.last().copied().expect("frames");
    assert!(transform.translation.z < -4.0, "{}", transform.translation);
    assert!(
        (soles(&transform) - RISE).abs() < 0.05,
        "{}",
        soles(&transform)
    );
    assert!(motion.grounded && motion.lag.length() < 0.01);
    let rises = seen.windows(2).map(|pair| pair[1].2.y - pair[0].2.y);
    let jump = rises.fold(0.0_f32, f32::max);
    assert!(
        jump > 0.0 && jump < 0.6 * RISE,
        "the camera rose {jump} m in a frame"
    );
}

/// A stair of ten steps laid out as the village builds them is climbed at a walk to its top,
/// at 30 frames a second and at 144.
#[test]
fn a_stair_is_climbed() {
    for fps in [30.0, 144.0] {
        let mut app = stage(fps);
        stair(&mut app, 1.5, 10);
        let seen = walk(&mut app, 1.0, (2.0 * fps) as usize);
        let (transform, motion, _) = seen.last().copied().expect("frames");
        let (top, end) = (10.0 * RISE, -(1.5 + 10.0 * RUN));
        let shown = format!(
            "{fps} fps: {} at {}",
            soles(&transform),
            transform.translation
        );
        assert!((soles(&transform) - top).abs() < 0.05, "{shown}");
        assert!(transform.translation.z < end && motion.grounded, "{shown}");
    }
}

/// Walking down the same stair the body never leaves the ground and ends on the floor.
#[test]
fn walking_down_a_stair_stays_grounded() {
    let mut app = stage(30.0);
    stair(&mut app, 1.5, 10);
    let top = FLOOR + 10.0 * RISE + SOLES + 0.02;
    put(&mut app, Vec3::new(0.0, top, -(1.5 + 10.0 * RUN + 1.0)));
    walk(&mut app, 0.0, 5);
    let seen = walk(&mut app, -1.0, 60);
    assert!(seen.iter().all(|frame| frame.1.grounded));
    let (transform, ..) = seen.last().copied().expect("frames");
    assert!(soles(&transform).abs() < 0.05, "{}", soles(&transform));
    assert!(transform.translation.z > 0.0, "{}", transform.translation);
}

/// A 0.5 m ledge, as tall as the side of a landed hull, stops the body: it neither climbs it
/// nor goes through it.
#[test]
fn a_tall_ledge_blocks() {
    let mut app = stage(30.0);
    block(&mut app, (1.5, 30.0), 0.5);
    let seen = walk(&mut app, 1.0, 60);
    for (transform, motion, _) in &seen {
        assert!(soles(transform) < 0.05, "{}", soles(transform));
        assert!(transform.translation.z > -1.5 + BODY_RADIUS - 0.02);
        assert!(motion.lag == Vec3::ZERO);
    }
}

/// A body in the air that walks into the side of a box whose top is just above its soles does
/// not climb onto it: it falls.
#[test]
fn no_step_in_the_air() {
    let mut app = stage(30.0);
    block(&mut app, (0.5, 30.0), 3.0);
    let start = FLOOR + 3.0 - 0.2 + SOLES;
    put(&mut app, Vec3::new(0.0, start, 0.0));
    let seen = walk(&mut app, 1.0, 20);
    assert!(seen.iter().all(|frame| frame.1.lag == Vec3::ZERO));
    let (transform, ..) = seen.last().copied().expect("frames");
    assert!(
        transform.translation.y < start - 1.0,
        "{}",
        transform.translation
    );
}
