//! The helm and the board computer: one of them owns the keyboard and the mouse, never both.

use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::mouse::MouseMotion;
use bevy::input::{ButtonState, InputPlugin};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::rig::settings;
use crate::computer::Computer;
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::helm::{Helm, read};
use crate::ship::pilot::Autopilot;
use crate::space::{Frame, Location, Travel};

/// Returns an app that only reads the helm of a ship in space, with the player in the seat
/// and the thrust key held.
fn cockpit() -> App {
    let dir = std::env::temp_dir().join(format!("abstractions-helm-{}", std::process::id()));
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, InputPlugin))
        .insert_resource(settings(&dir))
        .init_resource::<Helm>()
        .init_resource::<Autopilot>()
        .insert_resource(Location {
            body: default(),
            travel: Travel::InShip,
        })
        .init_state::<Computer>()
        .add_systems(Update, read);
    let ship = Ship::at_rest(Frame::System(default()), DVec3::ZERO, DQuat::IDENTITY);
    let world = app.world_mut();
    world.spawn(ship);
    world.write_message(KeyboardInput {
        key_code: KeyCode::KeyW,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app
}

/// Runs `frames` frames while the mouse keeps moving, and returns the helm and the throttle.
fn steer(app: &mut App, frames: usize) -> (Helm, f64) {
    for _ in 0..frames {
        let delta = Vec2::new(40.0, -25.0);
        app.world_mut().write_message(MouseMotion { delta });
        app.update();
    }
    let world = app.world_mut();
    let throttle = world.query::<&Ship>().single(world).unwrap().throttle;
    (*world.resource::<Helm>(), throttle)
}

/// With the computer closed the keys move the lever and the mouse moves the stick. While it
/// is open both are ignored: the stick is let go and the lever stays where it was. Closing it
/// gives the controls back.
#[test]
fn helm_lets_go_while_the_computer_is_open() {
    let mut app = cockpit();
    let (flying, pushed) = steer(&mut app, 5);
    assert!(pushed > 0.0 && flying.stick.length() > 0.1);
    assert_ne!(flying.controls, Controls::default());
    let mut next = app.world_mut().resource_mut::<NextState<Computer>>();
    next.set(Computer::Open);
    let (reading, held) = steer(&mut app, 5);
    assert_eq!(
        (reading.controls, reading.stick),
        (Controls::default(), Vec2::ZERO)
    );
    assert!(
        held >= pushed && held < pushed + 0.05,
        "the lever moved to {held}"
    );
    let mut next = app.world_mut().resource_mut::<NextState<Computer>>();
    next.set(Computer::Closed);
    let (back, lever) = steer(&mut app, 5);
    assert!(lever > held && back.stick.length() > 0.1);
}
