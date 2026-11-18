//! Warp sequence tests: the stages come in order, nothing ever steps, a warp cut short falls
//! back from where it is, and the frame rate does not change what plays.

use bevy::prelude::*;

use super::shader::streak;
use super::{DROP_TIME, Drive, ENGAGED, Phase, Sequence};

/// Seconds the drive of the script spools.
const SPOOL: f32 = 4.0;

/// Charge the drive of the script reports when it is fully spooled.
const SPOOLED: f32 = 0.45;

/// Returns what the drive of a scripted warp does `t` seconds in when it is switched off at
/// `cut`: it spools, engages, gathers speed for twenty seconds and holds it.
fn script(t: f32, cut: f32) -> Drive {
    let t = t + 1.0e-4;
    let speed = ((t - SPOOL) / 20.0).clamp(0.0, 1.0) * 0.7;
    match t {
        t if t >= cut => Drive::default(),
        t if t < SPOOL => Drive {
            charge: SPOOLED * t / SPOOL,
            speed: 0.0,
        },
        _ => Drive { charge: 1.0, speed },
    }
}

/// Plays the script cut at `cut` for `seconds` in frames of `dt` and returns every state.
fn play(dt: f32, cut: f32, seconds: f32) -> Vec<Sequence> {
    let frames = (seconds / dt).round() as usize;
    let mut state = Sequence::default();
    let step = |frame: usize| {
        state = state.advance(script(frame as f32 * dt, cut), dt);
        state
    };
    (0..frames).map(step).collect()
}

/// Returns the largest change between two frames of `states` of any value the sky, the view
/// and the engines draw from a state. The streak is the one of a near star, a light-year abeam.
fn largest_step(states: &[Sequence]) -> f32 {
    let drawn = |state: &Sequence| {
        let lens = state.lens(Vec3::NEG_Z);
        let felt = [lens.stars, state.tremor(), state.thrust()];
        let near = streak(lens.streak, 1.0, 1.0, 1.0);
        [lens.beta, near, lens.flash].into_iter().chain(felt)
    };
    let step = |pair: &[Sequence]| {
        let changes = drawn(&pair[0]).zip(drawn(&pair[1]));
        changes.map(|(a, b)| (a - b).abs()).fold(0.0, f32::max)
    };
    states.windows(2).map(step).fold(0.0, f32::max)
}

/// A whole warp goes through its stages in order, ends at rest, and no drawn value changes by
/// more than a few hundredths between two frames: no pop anywhere, no strobing.
#[test]
fn a_warp_runs_its_stages_in_order_without_a_step() {
    let states = play(1.0 / 60.0, 40.0, 44.0);
    let mut stages = vec![Phase::Idle];
    for state in &states {
        if stages.last() != Some(&state.phase) {
            stages.push(state.phase);
        }
        assert!((0.0..=1.0).contains(&state.progress) && (0.0..=1.0).contains(&state.charge));
    }
    let order = [
        Phase::Idle,
        Phase::Spool,
        Phase::Engage,
        Phase::Cruise,
        Phase::DropOut,
    ];
    assert_eq!(stages[..5], order);
    assert_eq!(stages[5..], [Phase::Idle]);
    assert_eq!(states.last(), Some(&Sequence::default()));
    assert!(largest_step(&states) < 0.04, "{}", largest_step(&states));
    let cruise = states[30 * 60];
    assert!(cruise.phase == Phase::Cruise && (cruise.velocity - 0.775).abs() < 1e-3);
}

/// While the drive spools the stars are points: the streak is exactly nothing. From the frame
/// it engages the streak grows from nothing, a little each frame.
#[test]
fn the_streak_grows_from_the_star_through_engage() {
    let dt = 1.0 / 60.0;
    let states = play(dt, 40.0, 10.0);
    let engage = (SPOOL / dt).round() as usize;
    assert!(states[..engage].iter().all(|state| state.streak() == 0.0));
    assert!(
        states[engage - 1].beta() > 0.1,
        "the sky gathers before it streaks"
    );
    let grown = &states[engage - 1..engage + 120];
    for pair in grown.windows(2) {
        let (was, now) = (pair[0].streak(), pair[1].streak());
        assert!(now >= was && now <= was * 1.25 + 1.0e-7, "{was} {now}");
    }
    let (first, later) = (states[engage].streak(), states[engage + 119].streak());
    assert!(
        first < 1.0e-8 && later > 2.0 * first && later > 0.0,
        "{first} {later}"
    );
    assert_eq!(states[engage].phase, Phase::Engage);
}

/// A warp stopped at any moment drops out from the state it is in: nothing steps at the cut,
/// the sky only opens again, and it is at rest within the time of a drop-out.
#[test]
fn an_early_drop_out_falls_back_from_where_it_is() {
    let dt = 1.0 / 60.0;
    for cut in [1.0, 3.9, 4.1, 4.5, 6.0, 30.0] {
        let states = play(dt, cut, cut + DROP_TIME + 0.2);
        let at = (cut / dt).round() as usize;
        assert_eq!(states[at].phase, Phase::DropOut, "cut at {cut}");
        assert!(
            largest_step(&states) < 0.04,
            "cut at {cut}: {}",
            largest_step(&states)
        );
        for pair in states[at - 1..].windows(2) {
            assert!(pair[1].beta() <= pair[0].beta() && pair[1].streak() <= pair[0].streak());
        }
        assert_eq!(states.last(), Some(&Sequence::default()), "cut at {cut}");
    }
    let blocked = play(dt, 3.0, 6.0);
    assert!(blocked.iter().all(|state| state.flash() == 0.0));
}

/// The flash of a drop-out rises from nothing over several frames, stays low and is gone in
/// well under a second after an arrival, when the ship flies at the speed of a fresh engage.
#[test]
fn the_flash_is_brief_and_soft() {
    let dt = 1.0 / 60.0;
    let arrival = Sequence {
        phase: Phase::Cruise,
        progress: 1.0,
        charge: 1.0,
        velocity: ENGAGED,
    };
    let mut state = arrival;
    let mut flashes = vec![state.flash()];
    for _ in 0..120 {
        state = state.advance(Drive::default(), dt);
        flashes.push(state.flash());
    }
    let peak = flashes.iter().copied().fold(0.0, f32::max);
    assert!(
        flashes[0] == 0.0 && flashes[1] == 0.0 && (0.05..=0.3).contains(&peak),
        "{peak}"
    );
    assert!(
        flashes
            .windows(2)
            .all(|pair| (pair[1] - pair[0]).abs() < 0.04)
    );
    assert!(
        flashes[30..].iter().all(|flash| *flash == 0.0),
        "{flashes:?}"
    );
    assert_eq!(state, Sequence::default());
}

/// Slow frames and a fast clock play the same sequence: the state at a given time is the same
/// at 240, 30 and 5 frames a second, and one huge frame lands on the state the drive asks for.
#[test]
fn the_frame_rate_does_not_change_the_sequence() {
    for seconds in [2.0, 4.4, 12.0, 30.0, 40.4, 41.0] {
        let at = |dt: f32| *play(dt, 40.0, seconds).last().unwrap();
        let (fine, coarse, slow) = (at(1.0 / 240.0), at(1.0 / 30.0), at(0.2));
        for other in [coarse, slow] {
            assert_eq!(other.phase, fine.phase, "{seconds}");
            assert!((other.charge - fine.charge).abs() < 0.03, "{seconds}");
            assert!((other.velocity - fine.velocity).abs() < 0.03, "{seconds}");
        }
    }
    let full = Drive {
        charge: 1.0,
        speed: 1.0,
    };
    let jumped = Sequence::default().advance(full, 1.0e3);
    assert_eq!((jumped.charge, jumped.velocity), (1.0, 1.0));
    assert_eq!(jumped.advance(Drive::default(), 1.0e3).phase, Phase::Idle);
    assert_eq!(jumped.advance(full, 0.0), jumped.advance(full, -1.0));
}
