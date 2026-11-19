//! Gaits: the duty factor falls with speed, and legs lift in the order of their gait.

use bevy::math::{Vec2, Vec3};

use super::feet::Foot;
use super::fixtures::quadruped_with;
use super::fixtures_legs::crawler;
use super::gait::{Mix, froude};
use super::lag::{Seat, blended, of};
use super::rig::Motion;
use super::tests_plans::drive;

/// Gravity on the test planet, metres per second squared.
const G: f32 = 9.81;

/// Returns the legs of `motion` that lifted since its feet were `before`.
fn lifted(before: &[Foot], motion: &Motion) -> Vec<usize> {
    let pairs = before.iter().zip(&motion.feet).enumerate();
    pairs
        .filter(|(_, (was, now))| was.planted && !now.planted)
        .map(|(leg, _)| leg)
        .collect()
}

/// Returns the lifts of a quadruped walking at Froude number `fr`, after it has settled: each
/// lift is the stride count at which it happened and the leg that lifted.
fn lifts(fr: f32) -> Vec<(f32, usize)> {
    let body = quadruped_with(2, 0.3);
    let speed = (fr * G * body.height).sqrt();
    let mut motion = Motion::new(&body, Vec3::ZERO, Vec3::NEG_Z, 3);
    let asked = drive(Vec3::NEG_Z * speed, 0.0);
    let (mut turns, mut lifts) = (0.0, Vec::new());
    for frame in 0..720 {
        let (before, phase) = (motion.feet.clone(), motion.clock.phase);
        motion.step(&body, &asked, &|_: Vec2| 0.0, 1.0 / 60.0);
        turns += f32::from(u8::from(motion.clock.phase < phase - 0.5));
        let at = turns + motion.clock.phase;
        lifts.extend(
            lifted(&before, &motion)
                .into_iter()
                .filter(|_| frame > 480)
                .map(|leg| (at, leg)),
        );
    }
    lifts
}

/// Returns `x` brought within half a stride of 0.
fn short(x: f32) -> f32 {
    x - x.round()
}

/// A quadruped at Fr 0.2 walks a lateral sequence: left hind, left fore, right hind, right fore,
/// each a quarter of a stride after the one before.
#[test]
fn quadruped_walks_a_lateral_sequence() {
    let lifts = lifts(0.2);
    let start = lifts.iter().position(|&(_, leg)| leg == 2).unwrap_or(0);
    let run = &lifts[start..start + 5];
    let legs: Vec<usize> = run.iter().map(|&(_, leg)| leg).collect();
    assert_eq!(legs, vec![2, 0, 3, 1, 2], "lifts {lifts:?}");
    for pair in run.windows(2) {
        assert!(
            (pair[1].0 - pair[0].0 - 0.25).abs() < 0.04,
            "gap {:?}",
            pair
        );
    }
}

/// A quadruped at Fr 1.5 trots: diagonal legs lift together, the two pairs half a stride apart.
#[test]
fn quadruped_trots_in_diagonal_pairs() {
    let lifts = lifts(1.5);
    let at = |leg: usize| {
        lifts
            .iter()
            .find(|&&(_, l)| l == leg)
            .map_or(f32::NAN, |&(t, _)| t)
    };
    let (lf, rf, lh, rh) = (at(0), at(1), at(2), at(3));
    assert!(
        short(lf - rh).abs() < 0.04,
        "fore left {lf} hind right {rh}"
    );
    assert!(
        short(rf - lh).abs() < 0.04,
        "fore right {rf} hind left {lh}"
    );
    assert!((short(lf - rf).abs() - 0.5).abs() < 0.04, "pairs {lf} {rf}");
}

/// The duty factor stays above a half at a walk, between 0.3 and 0.5 at a trot and under 0.4 at a gallop.
#[test]
fn duty_falls_with_speed() {
    for (fr, low, high) in [
        (0.1, 0.5, 1.0),
        (0.4, 0.5, 1.0),
        (1.0, 0.3, 0.5),
        (2.0, 0.3, 0.5),
        (3.5, 0.0, 0.4),
    ] {
        let duty = Mix::of(fr).duty(fr);
        assert!(duty > low && duty < high, "Fr {fr}: duty {duty}");
    }
    assert!((froude(2.0, 4.0, 1.0) - 1.0).abs() < 1e-6);
}

/// Hexapods run a tripod, long bodies a wave from back to front, and gaits blend without a jump.
#[test]
fn many_legs_keep_their_patterns() {
    let seat = |legs, ranks, rank, side| Seat {
        legs,
        ranks,
        rank,
        side,
    };
    assert_eq!(of(seat(6, 3, 0, -1), 0), of(seat(6, 3, 1, 1), 0));
    assert_eq!(of(seat(6, 3, 2, -1), 0), of(seat(6, 3, 0, -1), 0));
    assert_eq!(
        (of(seat(6, 3, 0, 1), 0) - of(seat(6, 3, 0, -1), 0)).abs(),
        0.5
    );
    let wave: Vec<f32> = (0..6).map(|rank| of(seat(12, 6, rank, -1), 0)).collect();
    assert!(
        wave.windows(2).all(|w| w[0] > w[1]),
        "back lifts first: {wave:?}"
    );
    let mut last = blended(seat(4, 2, 0, 1), &Mix::of(0.0));
    for step in 1..=400 {
        let lag = blended(seat(4, 2, 0, 1), &Mix::of(step as f32 * 0.01));
        assert!(
            short(lag - last).abs() < 0.05,
            "jump at Fr {}",
            step as f32 * 0.01
        );
        last = lag;
    }
    assert_eq!(crawler(6, 1.6).legs.len(), 12);
}
