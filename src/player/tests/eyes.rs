//! How the camera frames the body, and how the view and the keys aim it on the wings.

use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::stage::settings;
use crate::gear::{FlightState, Gear, Mode};
use crate::player::Player;
use crate::player::eyes::*;
use crate::settings::{Persistent, Settings};
use crate::view::{Heading, Kind, Look};

/// Wings out or a fast fall are framed as flying; walking, the boots and a slide as on foot.
#[test]
fn the_camera_frames_flight_and_fast_falls_as_flying() {
    let at = |mode: Mode, airspeed: f32| {
        kind(&FlightState {
            mode,
            airspeed,
            ..FlightState::default()
        })
    };
    assert_eq!(at(Mode::Glide, 5.0), Kind::Flying);
    assert_eq!(at(Mode::Fly, 60.0), Kind::Flying);
    assert_eq!(at(Mode::Fall, 40.0), Kind::Flying);
    assert_eq!(at(Mode::Fall, 6.0), Kind::OnFoot);
    assert_eq!(at(Mode::BootFall, 6.0), Kind::OnFoot);
    assert_eq!(at(Mode::Slide, 30.0), Kind::OnFoot);
    assert_eq!(at(Mode::Off, 30.0), Kind::OnFoot);
}

/// Returns a world with the local body gliding level at 30 m/s, the view level and dead
/// ahead, and nothing held.
fn gliding() -> World {
    let mut world = World::new();
    let state = FlightState {
        mode: Mode::Glide,
        wings: 1.0,
        velocity: Vec3::new(0.0, -7.0, -30.0),
        airspeed: 31.0,
        ..FlightState::default()
    };
    let gear = Gear {
        state,
        ..Gear::default()
    };
    world.spawn((Player, gear));
    world.insert_resource(settings());
    world.init_resource::<Time>();
    world.init_resource::<ButtonInput<KeyCode>>();
    world.init_resource::<ButtonInput<MouseButton>>();
    world.init_resource::<Heading>();
    world.init_resource::<Look>();
    world.init_resource::<WingAim>();
    world
}

/// Runs `frames` frames of a thirtieth of a second in which the mouse turns the view by
/// `mouse` radians, and returns the view the rig would hold and the aim.
fn frames(world: &mut World, frames: u32, mouse: Vec2) -> (Vec2, Vec2) {
    for _ in 0..frames {
        let step = Duration::from_secs_f64(1.0 / 30.0);
        world.resource_mut::<Time>().advance_by(step);
        *world.resource_mut::<Look>() = Look {
            yaw_delta: mouse.x,
            pitch_delta: mouse.y,
            ..Look::default()
        };
        world.run_system_once(steer).expect("the wings steer");
        let look = *world.resource::<Look>();
        let mut heading = world.resource_mut::<Heading>();
        heading.yaw += look.yaw_delta;
        heading.pitch += look.pitch_delta;
    }
    let heading = *world.resource::<Heading>();
    let view = Vec2::new(heading.yaw, heading.pitch);
    (view, world.resource::<WingAim>().stick.aim)
}

/// Holds or lets go of the steer button.
fn hold(world: &mut World, held: bool) {
    let mut buttons = world.resource_mut::<ButtonInput<MouseButton>>();
    match held {
        true => buttons.press(STEER),
        false => buttons.release(STEER),
    }
}

/// On the wings the mouse only looks around until the left button is held: then the aim goes
/// to the view, not in one frame, and it stays where it was when the button is let go.
#[test]
fn the_mouse_aims_the_wings_only_while_the_button_is_held() {
    let mut world = gliding();
    let (view, aim) = frames(&mut world, 15, Vec2::new(0.02, 0.01));
    assert!(view.x > 0.29 && view.y > 0.14, "the view stood: {view}");
    assert_eq!(aim, Vec2::ZERO, "the mouse steered with the button up");
    hold(&mut world, true);
    let (_, first) = frames(&mut world, 1, Vec2::ZERO);
    assert!(first.length() > 0.0 && first.length() < 0.05, "aim {first}");
    let (view, aim) = frames(&mut world, 60, Vec2::ZERO);
    assert_eq!(aim, view, "the aim did not reach the view");
    hold(&mut world, false);
    let (view, kept) = frames(&mut world, 30, Vec2::new(-0.02, 0.0));
    assert_eq!(kept, aim, "the aim moved after the release");
    assert!(view.x < aim.x - 0.5, "the view is not free: {view}");
}

/// The keys turn the aim and the view together with the button up, and a view left alone for
/// a while with the button up drifts back behind the body; one the button holds does not.
#[test]
fn the_keys_carry_the_view_and_an_idle_view_drifts_back() {
    let mut world = gliding();
    let back = world.resource::<Persistent<Settings>>().controls.keys.back;
    world.resource_mut::<ButtonInput<KeyCode>>().press(back);
    let (view, aim) = frames(&mut world, 15, Vec2::ZERO);
    assert!(aim.y > 0.4 && view == aim, "aim {aim}, view {view}");
    world.resource_mut::<ButtonInput<KeyCode>>().release(back);
    let (aside, _) = frames(&mut world, 10, Vec2::new(0.05, 0.0));
    let (soon, _) = frames(&mut world, 60, Vec2::ZERO);
    assert_eq!(soon, aside, "the view drifted back at once");
    let (later, kept) = frames(&mut world, 90, Vec2::ZERO);
    assert!(later.length() < 0.05, "the view stayed at {later}");
    assert_eq!(kept, aim, "the drift moved the aim");
    frames(&mut world, 10, Vec2::new(0.05, 0.0));
    hold(&mut world, true);
    let (held, _) = frames(&mut world, 120, Vec2::ZERO);
    assert!(held.x > 0.45, "a held view drifted back to {held}");
}

/// Off the wings the aim is the view, so that spreading them flies on toward where the
/// player looks.
#[test]
fn the_aim_is_the_view_until_the_wings_spread() {
    let mut world = gliding();
    let mut bodies = world.query::<&mut Gear>();
    for mut gear in bodies.iter_mut(&mut world) {
        gear.state.mode = Mode::Fall;
    }
    let (view, aim) = frames(&mut world, 5, Vec2::new(0.03, -0.02));
    assert!(
        view.x > 0.14 && aim.distance(view) < 1e-6,
        "{aim} for {view}"
    );
}
