//! Tests of the stow: opening the wingsuit or boarding slings a drawn tool over the back, the sling
//! runs its course back to ready, and a tool already slung or stowed is left alone.

use bevy::prelude::*;

use super::super::super::Avatar;
use super::super::act::{Act, Ask, Strike, Timing, next};
use super::super::dir::Dir;
use super::super::path::Path;
use super::super::stage::blade;
use super::stow::{self, stows};
use super::tune::STOW;
use crate::gear::{FlightState, Gear, GearInput, Mode as Flight, step};
use crate::player::{Motion, Player};
use crate::sim::PlayerId;

/// The wings asked open or the robot boarded sling a drawn tool, once; a stowed tool stays put.
#[test]
fn the_wings_or_the_ship_sling_a_drawn_tool() {
    assert!(stows(true, Act::Ready, (true, false)));
    assert!(stows(true, Act::Swing(0.1), (false, true)));
    assert!(!stows(true, Act::Ready, (false, false)));
    assert!(!stows(false, Act::Ready, (true, false)));
    assert!(!stows(true, Act::Stow(0.1), (true, false)));
}

/// The sling takes its time and comes back to ready, where the hands let the tool go.
#[test]
fn the_sling_runs_back_to_ready() {
    let held = blade();
    let timing = Timing::of(&held, &Path::of(held.style, Dir::Right));
    let (mut act, mut seconds) = (Act::Stow(0.0), 0.0);
    while matches!(act, Act::Stow(_)) && seconds < 2.0 {
        act = next(act, &Ask::default(), &timing, false, 1.0 / 60.0).0;
        seconds += 1.0 / 60.0;
    }
    assert_eq!(act, Act::Ready);
    assert!((seconds - STOW).abs() < 0.05, "{seconds}");
}

/// Opening the wingsuit slings the tool of the local robot.
#[test]
fn opening_the_wingsuit_stows_the_tool() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, stow::stow);
    let motion = Motion {
        wings: true,
        ..default()
    };
    let player = app
        .world_mut()
        .spawn((Player, PlayerId::LOCAL, motion, Gear::default()))
        .id();
    let body = app
        .world_mut()
        .spawn((Avatar, blade(), ChildOf(player)))
        .id();
    app.update();
    let strike = *app.world().get::<Strike>(body).expect("a strike");
    assert_eq!(strike.act, Act::Stow(0.0));
}

/// The sling is the first phase of the opening: run frame by frame beside a gear whose wings are
/// asked open as the controller asks them once [`crate::player::SLING`] has run, the tool is in
/// the hand only while the wing is folded, and never once the arms spread.
#[test]
fn the_tool_leaves_the_hand_before_the_wings_spread() {
    let held = blade();
    let timing = Timing::of(&held, &Path::of(held.style, Dir::Right));
    let dt = 1.0 / 60.0;
    let (mut act, mut out) = (Act::Stow(0.0), true);
    let mut state = FlightState {
        mode: Flight::Fall,
        velocity: Vec3::new(0.0, -10.0, -8.0),
        ..FlightState::default()
    };
    for frame in 0..60 {
        let t = frame as f32 * dt;
        let input = GearInput {
            deploy: t >= crate::player::SLING,
            ..GearInput::default()
        };
        state = step(&state, &input, dt);
        let stowing = matches!(act, Act::Stow(_));
        act = next(act, &Ask::default(), &timing, false, dt).0;
        out &= !(stowing && !matches!(act, Act::Stow(_)));
        assert!(
            !out || state.spread() == 0.0,
            "frame {frame}: tool in a spread wing"
        );
    }
    assert!(
        !out && state.spread() > 0.5,
        "out {out}, spread {}",
        state.spread()
    );
}
