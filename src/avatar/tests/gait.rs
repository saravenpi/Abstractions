//! The leg cycle: feet that do not slide, a cadence that rises with speed, and a leg solver that reaches.

use bevy::prelude::*;

use super::gap;
use crate::avatar::clock::Clock;
use crate::avatar::gait::{advance, duty, foot, leg_ik, stride_length};
use crate::avatar::ground::stride;
use crate::avatar::rig::{Seg, globals};
use crate::avatar::tune::{ANKLE_RISE, SHIN, SOLE_Y, THIGH};
use crate::avatar::{AvatarPose, Mode};

/// Speeds the gait is checked at, in meters per second: a crawl, walks, the run threshold, a sprint.
const SPEEDS: [f32; 6] = [0.6, 1.5, 3.0, 5.0, 6.5, 8.5];

/// Simulation steps per stride.
const STEPS: usize = 600;

/// Returns the largest drift of a planted ankle over two strides at `speed`: along the ground, and in height.
fn drift(speed: f32) -> (f32, f32) {
    let input = AvatarPose {
        mode: Mode::Run,
        speed,
        ..default()
    };
    let step = stride_length(speed) / STEPS as f32;
    let (mut clock, mut worst) = (Clock::default(), (0.0_f32, 0.0_f32));
    let mut planted: [Option<f32>; 2] = [None; 2];
    for tick in 0..2 * STEPS {
        clock.phase = advance(clock.phase, step, speed);
        let frames = globals(&stride(&input, &clock));
        for (side, boot) in [Seg::BootL, Seg::BootR].into_iter().enumerate() {
            let ankle = frames[boot as usize].translation;
            let along = ankle.z - step * (tick + 1) as f32;
            if !foot(clock.phase + 0.5 * side as f32, speed).planted {
                planted[side] = None;
                continue;
            }
            let start = *planted[side].get_or_insert(along);
            let tilt = (frames[boot as usize].rotation * Vec3::Y).angle_between(Vec3::Y);
            assert!(tilt < 0.02, "sole tilted by {tilt} at {speed} m/s");
            worst = (
                worst.0.max((along - start).abs()),
                worst.1.max((ankle.y - SOLE_Y - ANKLE_RISE).abs()),
            );
        }
    }
    worst
}

/// A planted foot stays where it was put, within two centimeters, over whole strides at every speed.
#[test]
fn planted_feet_do_not_slide() {
    for speed in SPEEDS {
        let (along, height) = drift(speed);
        assert!(along < 0.02, "foot slid {along} m at {speed} m/s");
        assert!(
            height < 0.02,
            "foot left the ground by {height} m at {speed} m/s"
        );
    }
}

/// Each foot is planted for its share of the stride, one after the other, and the cadence rises with speed.
#[test]
fn cadence_rises_with_speed() {
    let cadence = |speed: f32| speed / stride_length(speed);
    for pair in SPEEDS.windows(2) {
        assert!(
            cadence(pair[1]) > cadence(pair[0]),
            "cadence falls at {} m/s",
            pair[1]
        );
        assert!(stride_length(pair[1]) >= stride_length(pair[0]));
    }
    assert!(
        cadence(8.5) > cadence(5.0) && cadence(8.5) < 4.0,
        "sprint cadence {}",
        cadence(8.5)
    );
    for speed in SPEEDS {
        let share = duty(speed);
        assert!(share > 0.05 && share <= 0.5, "duty {share} at {speed}");
        let planted = (0..1000)
            .filter(|i| foot(*i as f32 / 1000.0, speed).planted)
            .count();
        assert!((planted as f32 / 1000.0 - share).abs() < 0.01);
        let both = (0..1000).filter(|i| {
            let at = *i as f32 / 1000.0;
            foot(at, speed).planted && foot(at + 0.5, speed).planted
        });
        assert!(
            both.count() <= 2,
            "both feet planted at once at {speed} m/s"
        );
    }
}

/// The leg solver puts the ankle on a reachable target with the knee bending forward only.
#[test]
fn leg_solver_reaches_its_target() {
    for (ahead, down) in [
        (0.0, 0.7),
        (0.2, 0.6),
        (-0.25, 0.55),
        (0.3, 0.3),
        (0.0, 0.2),
        (-0.1, 0.71),
    ] {
        let (hip, knee) = leg_ik(ahead, down);
        assert!((0.0..=3.0).contains(&knee), "knee {knee}");
        let reached = Vec2::new(
            THIGH * hip.sin() + SHIN * (hip - knee).sin(),
            THIGH * hip.cos() + SHIN * (hip - knee).cos(),
        );
        assert!(
            reached.distance(Vec2::new(ahead, down)) < 2e-3,
            "missed by {reached}"
        );
    }
    let (hip, knee) = leg_ik(0.0, 5.0);
    assert!(hip.is_finite() && knee.is_finite() && knee >= 0.0);
}

/// The cycle closes: the pose at the end of a stride is the pose at its start, and a foot never jumps.
#[test]
fn the_cycle_closes() {
    for speed in SPEEDS {
        let input = AvatarPose {
            mode: Mode::Walk,
            speed,
            ..default()
        };
        let at = |phase: f32| stride(&input, &Clock { phase, ..default() });
        assert!(
            gap(&at(0.99999), &at(0.00001)) < 0.01,
            "seam at {speed} m/s"
        );
        for i in 0..2000 {
            let (a, b) = (
                foot(i as f32 / 2000.0, speed),
                foot((i + 1) as f32 / 2000.0, speed),
            );
            assert!((a.ahead - b.ahead).abs() < 0.01 && (a.lift - b.lift).abs() < 0.01);
        }
    }
}
