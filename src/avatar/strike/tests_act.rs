//! Tests of the acts: the timing of a swing from the build, the guard and its parry window, the draw
//! and release of a spear, the style of each build and the direction read from the mouse.

use bevy::prelude::*;

use super::act::{Act, Ask, Strike, Timing, key, next};
use super::dir::{Dir, Strokes};
use super::held::{Held, Style};
use super::path::Path;
use super::stage::{blade, club, knife, spear};
use super::tune::{DRAW, PARRY, PARRY_LIGHT, RECOVER};

/// Seconds in a frame at sixty frames per second.
const FRAME: f32 = 1.0 / 60.0;

/// Runs `act` under `ask` for `seconds` in frames and returns where it ends and any draw let go.
fn run(mut act: Act, ask: &Ask, timing: &Timing, thrown: bool, seconds: f32) -> (Act, Option<f32>) {
    let mut loosed = None;
    for _ in 0..(seconds / FRAME).round() as usize {
        let (after, let_go) = next(act, ask, timing, thrown, FRAME);
        act = after;
        loosed = loosed.or(let_go);
    }
    (act, loosed)
}

/// A click winds up for 2·arc / ω, swings through the arc at ω, recovers and is ready again.
#[test]
fn a_swing_follows_the_build() {
    let held = blade();
    let timing = Timing::of(&held, &Path::of(held.style, Dir::Right));
    assert!((timing.windup - held.swing.windup).abs() < 1e-6);
    assert!(
        (timing.swing - Path::of(Style::Slash, Dir::Right).length() / held.swing.spin).abs() < 1e-6
    );
    let click = Ask {
        strike: true,
        holding: true,
        ..default()
    };
    let (act, _) = next(Act::Ready, &click, &timing, false, FRAME);
    assert_eq!(act, Act::Windup(0.0));
    let rest = Ask::default();
    let (act, _) = run(act, &rest, &timing, false, timing.windup + FRAME);
    assert!(matches!(act, Act::Swing(_)), "{act:?}");
    let (act, _) = run(act, &rest, &timing, false, timing.swing + FRAME);
    assert!(matches!(act, Act::Recover(_)), "{act:?}");
    let (act, _) = run(act, &rest, &timing, false, timing.recover + FRAME);
    assert_eq!(act, Act::Ready);
}

/// A heavier head swings slower and winds up longer.
#[test]
fn a_heavy_head_swings_slower() {
    let (light, heavy) = (blade(), club());
    assert!(heavy.swing.spin < light.swing.spin);
    assert!(heavy.swing.windup > light.swing.windup);
}

/// A guard holds while the button is held and parries only in its first 150 ms, 100 ms with a
/// very light tool.
#[test]
fn a_guard_parries_at_first() {
    let held = club();
    let timing = Timing::of(&held, &Path::of(held.style, Dir::Up));
    let guard = Ask {
        guard: true,
        ..default()
    };
    let (act, _) = run(Act::Ready, &guard, &timing, false, 0.1);
    let strike = Strike {
        act,
        dir: Dir::Left,
        ..default()
    };
    assert_eq!(strike.guarding(), Some(Dir::Left));
    assert!(strike.parries(held.parry()));
    let (act, _) = run(act, &guard, &timing, false, 0.2);
    assert!(!Strike { act, ..default() }.parries(held.parry()));
    assert_eq!(
        next(act, &Ask::default(), &timing, false, FRAME).0,
        Act::Ready
    );
    assert_eq!((held.parry(), knife().parry()), (PARRY, PARRY_LIGHT));
}

/// A spear draws while the button is held and lets go on release, with the share of a full draw.
#[test]
fn a_spear_draws_and_throws() {
    let held = spear();
    let timing = Timing::of(&held, &Path::of(held.style, Dir::Right));
    let hold = Ask {
        strike: true,
        holding: true,
        ..default()
    };
    let (act, _) = next(Act::Ready, &hold, &timing, true, FRAME);
    assert_eq!(act, Act::Draw(0.0));
    let (act, _) = run(
        act,
        &Ask {
            holding: true,
            ..default()
        },
        &timing,
        true,
        DRAW / 2.0,
    );
    let (act, loosed) = next(act, &Ask::default(), &timing, true, FRAME);
    assert!(matches!(act, Act::Recover(_)));
    let share = loosed.expect("a half draw throws");
    assert!((share - 0.5).abs() < 0.05, "{share}");
    let (short, loosed) = next(Act::Draw(0.05), &Ask::default(), &timing, true, FRAME);
    assert_eq!((short, loosed), (Act::Ready, None));
}

/// A point thrusts, a heavy head chops and comes overhead when asked from below, an edge slashes.
#[test]
fn each_build_has_its_style() {
    assert_eq!(
        (spear().style, club().style, blade().style),
        (Style::Thrust, Style::Chop, Style::Slash)
    );
    assert!(matches!(
        Path::of(Style::Thrust, Dir::Left),
        Path::Line { .. }
    ));
    assert_eq!(
        Path::of(Style::Chop, Dir::Down),
        Path::of(Style::Chop, Dir::Up)
    );
    assert_ne!(
        Path::of(Style::Slash, Dir::Down),
        Path::of(Style::Slash, Dir::Up)
    );
}

/// The mouse motion of the last 120 ms picks the side, and a resting mouse keeps the last one.
#[test]
fn the_mouse_picks_the_side() {
    let mut strokes = Strokes::default();
    strokes.push(0.0, Vec2::new(-40.0, 0.0));
    assert_eq!(strokes.dir(0.05), Dir::Left);
    strokes.push(1.0, Vec2::new(2.0, -30.0));
    assert_eq!(strokes.dir(1.05), Dir::Up);
    assert_eq!(strokes.dir(3.0), Dir::Up);
    strokes.push(4.0, Vec2::new(0.0, 25.0));
    strokes.push(4.05, Vec2::new(50.0, 0.0));
    assert_eq!(strokes.dir(4.1), Dir::Right);
}

/// A heavier head recovers for longer than a light one, never under the least, and carries on further
/// past the follow-through; the overshoot settles back to the carried tool.
#[test]
fn a_heavy_head_recovers_longer_and_carries_on() {
    let path = |h: &Held| Path::of(h.style, Dir::Right);
    let (light, heavy) = (blade(), club());
    let (a, b) = (
        Timing::of(&light, &path(&light)),
        Timing::of(&heavy, &path(&heavy)),
    );
    assert!(a.recover >= RECOVER && b.recover > a.recover);
    assert!(b.over > a.over && b.over < 0.2);
    let p = path(&heavy);
    let mid = key(Act::Recover(0.5 * b.recover), &p, &b, Dir::Right);
    assert!(mid.grip.distance(p.at(1.0).grip) > 1e-4);
    let end = key(Act::Recover(b.recover), &p, &b, Dir::Right);
    assert!(end.grip.distance(super::key::carry().grip) < 1e-4);
}
