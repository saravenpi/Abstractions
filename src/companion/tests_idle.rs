//! Tests of the idle picker, the poses and the idle mind.

use bevy::prelude::*;

use super::idle::{Idle, WEIGHTS, pick};
use super::mind::{Cue, Input, Mind};
use super::pose::{doze_pose, idle_pose};
use super::rng::Rng;
use super::tune::{DOZE_AFTER, IDLE_AFTER};

/// Returns the input of one sixtieth of a second for a player moving at `speed`.
fn frame(speed: f32) -> Input {
    Input {
        speed,
        night: false,
        seated: false,
        dt: 1.0 / 60.0,
    }
}

/// The same seed gives the same idles, and a different seed a different run.
#[test]
fn idle_picker_is_deterministic() {
    let run = |seed: u64| {
        let mut rng = Rng::new(seed);
        let mut last = None;
        (0..200)
            .map(|_| {
                let idle = pick(&mut rng, last, true);
                last = Some(idle);
                idle
            })
            .collect::<Vec<Idle>>()
    };
    assert_eq!(run(9), run(9));
    assert_ne!(run(9), run(10));
}

/// Over ten thousand draws each idle comes up as often as its weight says, the sky only at night,
/// and never the same idle twice in a row.
#[test]
fn idle_picker_honours_weights() {
    let mut rng = Rng::new(3);
    let total: f32 = WEIGHTS.iter().map(|(_, weight)| weight).sum();
    let draws: Vec<Idle> = (0..10_000).map(|_| pick(&mut rng, None, true)).collect();
    for (idle, weight) in WEIGHTS {
        let share = draws.iter().filter(|drawn| **drawn == idle).count() as f32 / 10_000.0;
        assert!((share - weight / total).abs() < 0.02, "{idle:?} {share}");
    }
    let mut last = None;
    for _ in 0..10_000 {
        let idle = pick(&mut rng, last, false);
        assert!(Some(idle) != last && idle != Idle::SkyPeek);
        last = Some(idle);
    }
}

/// Every idle starts and ends with the body at rest, and the doze sinks it.
#[test]
fn poses_start_and_end_at_rest() {
    for (idle, _) in WEIGHTS {
        let len = idle.seconds();
        for t in [0.0, len] {
            let pose = idle_pose(idle, t, len);
            assert!(pose.rotation().angle_between(Quat::IDENTITY) < 1e-3);
            assert!(pose.lift.abs() + pose.spread + pose.wave < 1e-3, "{idle:?}");
        }
        let busy = (1..20).any(|i| idle_pose(idle, len * i as f32 / 20.0, len) != default());
        assert!(busy, "{idle:?} does nothing");
    }
    assert!(doze_pose(10.0).lift < -0.15);
}

/// Idles start once the player has stood still for a while, follow each other without repeating
/// and stop at once when the player moves or sits in the ship.
#[test]
fn idles_run_only_while_standing_still() {
    let mut mind = Mind::new(11);
    let mut started = Vec::new();
    for tick in 0..(40 * 60) {
        let cue = mind.advance(&frame(0.0));
        if let Some(Cue::Started(idle)) = cue {
            assert!(tick as f32 / 60.0 >= IDLE_AFTER - 0.05);
            started.push(idle);
        }
    }
    assert!(started.len() >= 3, "only {started:?}");
    assert!(started.windows(2).all(|pair| pair[0] != pair[1]));
    assert_eq!(mind.advance(&frame(3.0)), None);
    assert!(mind.idle.is_none() && mind.pose() == default());
    let seated = Input {
        seated: true,
        ..frame(0.0)
    };
    for _ in 0..(120 * 60) {
        assert_eq!(mind.advance(&seated), None);
    }
    assert!(!mind.dozing);
}

/// After a long time standing still it dozes, stays asleep, and wakes the moment the player moves.
#[test]
fn dozes_after_a_long_idle_and_wakes_on_movement() {
    let mut mind = Mind::new(5);
    let mut cues = Vec::new();
    for _ in 0..((DOZE_AFTER + 20.0) * 60.0) as usize {
        cues.extend(mind.advance(&frame(0.05)));
    }
    assert_eq!(cues.last(), Some(&Cue::Dozed));
    assert_eq!(cues.iter().filter(|cue| **cue == Cue::Dozed).count(), 1);
    assert!(mind.dozing && mind.idle.is_none());
    assert!(mind.pose().lift < -0.1, "it should sink while dozing");
    assert_eq!(mind.advance(&frame(2.0)), Some(Cue::Woke));
    assert!(!mind.dozing && mind.still == 0.0);
    assert_eq!(mind.advance(&frame(2.0)), None);
}
