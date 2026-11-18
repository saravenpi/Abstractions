//! The helm and the board computer: one of them owns the keyboard and the mouse, never both.

use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::{ButtonState, InputPlugin};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::rig::settings;
use crate::computer::Computer;
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::helm::mouse::{STEER, STICK_PER_PIXEL, deflection, stick};
use crate::ship::helm::{Helm, read, steerable};
use crate::ship::pilot::Autopilot;
use crate::ship::warp::Warp;
use crate::space::{Frame, Location, Travel};

/// Returns an app that only reads the helm of a ship in space, with the player in the seat
/// and the thrust key held.
fn cockpit() -> App {
    let mut app = seat();
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::KeyW,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app
}

/// Returns an app that only reads the helm of a ship in space, with the player in the seat
/// and no key held.
fn seat() -> App {
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
    app.world_mut().spawn(ship);
    app
}

/// Runs `frames` frames while the mouse keeps moving, and returns the helm and how far the
/// held key has pushed the nose down.
fn steer(app: &mut App, frames: usize) -> (Helm, f64) {
    for _ in 0..frames {
        let delta = Vec2::new(40.0, -25.0);
        app.world_mut().write_message(MouseMotion { delta });
        app.update();
    }
    let world = app.world_mut();
    let helm = *world.resource::<Helm>();
    (helm, -helm.controls.turn.x)
}

/// With the computer closed the keys steer the ship and the mouse steers nothing: the forward
/// key pushes the nose down and nothing rolls. While the computer is open the keys are
/// ignored. Closing it gives the controls back.
#[test]
fn helm_lets_go_while_the_computer_is_open() {
    let mut app = cockpit();
    let (flying, pushed) = steer(&mut app, 5);
    assert!(pushed > 0.99 && flying.controls.turn.z == 0.0 && flying.controls.turn.y == 0.0);
    let mut next = app.world_mut().resource_mut::<NextState<Computer>>();
    next.set(Computer::Open);
    let (reading, held) = steer(&mut app, 5);
    assert_eq!((reading.controls, held), (Controls::default(), 0.0));
    let mut next = app.world_mut().resource_mut::<NextState<Computer>>();
    next.set(Computer::Closed);
    let (_, back) = steer(&mut app, 5);
    assert!(back > 0.99);
}

/// An engaged warp takes no steering, whatever the keys or the mouse ask; off or spooling, they steer.
#[test]
fn an_engaged_warp_cannot_be_steered() {
    let keys = DVec3::new(1.0, -1.0, 0.5);
    assert_eq!(steerable(Warp::Active { speed: 2.0e9 }, keys), DVec3::ZERO);
    assert_eq!(steerable(Warp::Off, keys), keys);
    assert_eq!(steerable(Warp::Spooling { left: 2.0 }, keys), keys);
}

/// Presses or lets go of the button that hands the mouse to the hull.
fn hold_button(app: &mut App, state: ButtonState) {
    app.world_mut().write_message(MouseButtonInput {
        button: STEER,
        state,
        window: Entity::PLACEHOLDER,
    });
}

/// With the left button up a moving mouse asks nothing of the hull. While it is held the nose
/// follows the mouse, to the right and up here, by exactly the stick rule, and nothing rolls.
/// Letting go centres the stick at once: the hull keeps the attitude it has.
#[test]
fn the_mouse_steers_only_while_its_left_button_is_held() {
    let mut app = seat();
    let (free, _) = steer(&mut app, 5);
    assert_eq!((free.controls.turn, free.stick), (DVec3::ZERO, Vec2::ZERO));
    hold_button(&mut app, ButtonState::Pressed);
    let (held, _) = steer(&mut app, 1);
    let nudge = Vec2::new(40.0, -25.0) * STICK_PER_PIXEL;
    let dt = app.world().resource::<Time>().delta_secs();
    let wanted = stick(Vec2::ZERO, nudge, true, dt);
    assert_eq!(held.stick, wanted);
    assert_eq!(held.controls.turn, deflection(wanted));
    let turn = held.controls.turn;
    assert!(turn.x > 0.05 && turn.y < -0.1 && turn.z == 0.0, "{turn}");
    let (long, _) = steer(&mut app, 60);
    assert!(long.stick.length() <= 1.0 && long.controls.turn.y < turn.y);
    hold_button(&mut app, ButtonState::Released);
    let (let_go, _) = steer(&mut app, 1);
    assert_eq!(
        (let_go.controls.turn, let_go.stick),
        (DVec3::ZERO, Vec2::ZERO)
    );
}

/// The mouse stick moves only while the button is held, never leaves the unit disc, and drifts
/// back to the centre while the mouse rests. In an engaged warp a held button steers nothing.
#[test]
fn the_mouse_stick_is_bounded_and_returns_to_the_centre() {
    let far = stick(Vec2::ZERO, Vec2::new(5.0, -3.0), true, 0.016);
    assert!((far.length() - 1.0).abs() < 1e-6);
    assert_eq!(stick(far, Vec2::new(5.0, 0.0), false, 0.016), Vec2::ZERO);
    let mut resting = far;
    for _ in 0..120 {
        resting = stick(resting, Vec2::ZERO, true, 1.0 / 60.0);
    }
    assert!(resting.length() < 0.01, "{resting}");
    let mut app = seat();
    let mut ships = app.world_mut().query::<&mut Ship>();
    ships.single_mut(app.world_mut()).unwrap().warp = Warp::Active { speed: 2.0e9 };
    hold_button(&mut app, ButtonState::Pressed);
    let (warping, _) = steer(&mut app, 5);
    assert_eq!(warping.controls.turn, DVec3::ZERO);
}
