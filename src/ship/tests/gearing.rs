//! The landing gear folds above 40 m and comes down under 25 m, rising ground lifts a hovering
//! ship, and the keys still do what they did in 0.8.1, easing in over a planet.

use std::time::Duration;

use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::{ButtonState, InputPlugin};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::hover::{assisted, stick};
use super::rig::settings;
use super::{coast, fly, planet};
use crate::computer::Computer;
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::gear::{HEIGHTS, TRAVEL, stow};
use crate::ship::helm::{Helm, read};
use crate::ship::landing::GEAR_DROP;
use crate::ship::pilot::Autopilot;
use crate::space::{Frame, Location, Travel};

/// On an approach that bumps 5 m up and down while it comes down from 60 m, the gear goes up
/// once above 40 m and comes down once under 25 m, never more.
#[test]
fn the_gear_has_two_heights_and_never_cycles() {
    let mut ship = Ship::at_rest(Frame::System(default()), DVec3::ZERO, DQuat::IDENTITY);
    let (mut flips, mut was) = (0, ship.stowed);
    let path = (0..1200).map(|i| f64::from(i) * 0.1).map(|t| {
        let climb = if t < 30.0 {
            2.0 * t
        } else {
            60.0 - 0.7 * (t - 30.0)
        };
        climb + 5.0 * (t * 3.0).sin()
    });
    for radar in path {
        stow(&mut ship, radar.max(0.0), 0.1);
        flips += usize::from(ship.stowed != was);
        was = ship.stowed;
    }
    assert_eq!((flips, ship.stowed, ship.gear), (2, false, 1.0));
    assert!(HEIGHTS.0 > HEIGHTS.1 + 10.0);
}

/// A ship that rises from 20 m on the rise key folds its gear within the travel time of
/// passing 40 m, and sinking back it is down and locked by the time it passes 25 m minus the
/// travel at the sink rate.
#[test]
fn the_gear_folds_on_the_way_up_and_comes_down_to_land() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
    let mut ship = assisted(&planet, 20.0);
    let mut folded_at = None;
    fly(&mut ship, &stick(1.0), land, 9.0, |ship, report| {
        folded_at = folded_at.or((ship.gear == 0.0).then_some(report.radar));
    });
    let up = folded_at.expect("the gear folded");
    assert!(
        ship.stowed && up > HEIGHTS.0 && up < HEIGHTS.0 + 10.0 * TRAVEL + 5.0,
        "{up}"
    );
    let mut locked_at = None;
    fly(&mut ship, &stick(-1.0), land, 20.0, |ship, report| {
        locked_at = locked_at.or((ship.gear == 1.0).then_some(report.radar));
    });
    let down = locked_at.expect("the gear came down");
    assert!(
        down < HEIGHTS.1 && down > HEIGHTS.1 - 6.0 * TRAVEL - 2.0,
        "{down}"
    );
    coast(&mut ship, &stick(-1.0), land, 10.0);
    assert!(ship.landed && ship.gear == 1.0);
}

/// Ground that rises under a hovering ship lifts it: hovering 2 m over flat ground that then
/// rises by 1.5 m, the gear never touches it and climbs back above it.
#[test]
fn rising_ground_lifts_a_hovering_ship() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = assisted(&planet, GEAR_DROP + 2.0);
    coast(&mut ship, &Controls::default(), (&planet, &|_| 0.0), 2.0);
    let mut touched = false;
    fly(
        &mut ship,
        &Controls::default(),
        (&planet, &|_| 1.5),
        5.0,
        |_, report| {
            touched |= report.radar <= 0.0;
        },
    );
    let radar = ship.pos.length() - planet.radius - 1.5 - GEAR_DROP;
    assert!(!touched && radar > 1.3, "gear {radar} m over the ground");
}

/// Returns an app that only reads the helm, with the player seated in a ship in `frame`.
fn cockpit(frame: Frame) -> App {
    let name = format!("abstractions-keys-{}", std::process::id());
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, InputPlugin))
        .insert_resource(settings(&std::env::temp_dir().join(name)))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .init_resource::<Helm>()
        .init_resource::<Autopilot>()
        .insert_resource(Location {
            body: default(),
            travel: Travel::InShip,
        })
        .init_state::<Computer>()
        .add_systems(Update, read);
    app.world_mut()
        .spawn(Ship::at_rest(frame, DVec3::Y * 7000.0, DQuat::IDENTITY));
    app
}

/// Holds `key` for `frames` frames of 50 ms, returns the stick and the vertical stick, and lets go.
fn hold(app: &mut App, key: KeyCode, frames: usize) -> (DVec3, f64) {
    let press = |state: ButtonState| KeyboardInput {
        key_code: key,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    };
    app.world_mut().write_message(press(ButtonState::Pressed));
    (0..frames).for_each(|_| app.update());
    let controls = app.world().resource::<Helm>().controls;
    app.world_mut().write_message(press(ButtonState::Released));
    app.update();
    (controls.turn, controls.lift)
}

/// Every turn and vertical key moves the same axis the same way as in 0.8.1: W nose down, S
/// nose up, Q left, E right, A roll left, D roll right, Space up, C down. Held for a second
/// each is at full deflection over a planet; a 100 ms tap there gives under a tenth of it.
#[test]
fn every_key_moves_the_ship_as_in_0_8_1() {
    let mut app = cockpit(Frame::Surface(default()));
    let turns = [
        (KeyCode::KeyW, DVec3::NEG_X),
        (KeyCode::KeyS, DVec3::X),
        (KeyCode::KeyQ, DVec3::Y),
        (KeyCode::KeyE, DVec3::NEG_Y),
        (KeyCode::KeyA, DVec3::Z),
        (KeyCode::KeyD, DVec3::NEG_Z),
    ];
    for (key, axis) in turns {
        assert_eq!(hold(&mut app, key, 20), (axis, 0.0), "{key:?}");
        let (tap, _) = hold(&mut app, key, 2);
        assert!(tap.dot(axis) > 0.0 && tap.length() < 0.1, "{key:?}: {tap}");
    }
    for (key, lift) in [(KeyCode::Space, 1.0), (KeyCode::KeyC, -1.0)] {
        assert_eq!(hold(&mut app, key, 1), (DVec3::ZERO, lift), "{key:?}");
    }
    let mut space = cockpit(Frame::System(default()));
    assert_eq!(hold(&mut space, KeyCode::KeyW, 1).0, DVec3::NEG_X);
}
