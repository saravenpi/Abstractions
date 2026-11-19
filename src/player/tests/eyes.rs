//! How the camera frames the body, and how the view chases it on the wings.

use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::stage::settings;
use crate::gear::{FlightState, Gear, Mode};
use crate::player::Player;
use crate::player::eyes::*;
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
pub(super) fn gliding() -> World {
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
    world.init_resource::<WingView>();
    world
}

/// Runs `frames` frames of a thirtieth of a second in which the mouse turns the view by
/// `mouse` radians and the nose of the body turns by `nose`, and returns the view the rig
/// would hold.
fn frames(world: &mut World, frames: u32, (mouse, nose): (Vec2, f32)) -> Vec2 {
    for _ in 0..frames {
        let step = Duration::from_secs_f64(1.0 / 30.0);
        world.resource_mut::<Time>().advance_by(step);
        let mut bodies = world.query::<&mut Gear>();
        for mut gear in bodies.iter_mut(world) {
            gear.state.servo.yaw += nose;
        }
        *world.resource_mut::<Look>() = Look {
            yaw_delta: mouse.x,
            pitch_delta: mouse.y,
            ..Look::default()
        };
        world.run_system_once(steer).expect("the view turns");
        let look = *world.resource::<Look>();
        let mut heading = world.resource_mut::<Heading>();
        heading.yaw += look.yaw_delta;
        heading.pitch += look.pitch_delta;
    }
    let heading = *world.resource::<Heading>();
    Vec2::new(heading.yaw, heading.pitch)
}

/// Holds or lets go of the steer button.
fn hold(world: &mut World, held: bool) {
    let mut buttons = world.resource_mut::<ButtonInput<MouseButton>>();
    match held {
        true => buttons.press(STEER),
        false => buttons.release(STEER),
    }
}

/// On the wings the mouse looks around freely, and a view left alone for a while drifts back
/// behind the body; one the steer button holds does not.
#[test]
fn the_mouse_looks_around_and_an_idle_view_drifts_back() {
    let mut world = gliding();
    let aside = frames(&mut world, 15, (Vec2::new(0.02, 0.01), 0.0));
    assert!(aside.x > 0.29 && aside.y > 0.14, "the view stood: {aside}");
    let soon = frames(&mut world, 60, (Vec2::ZERO, 0.0));
    assert_eq!(soon, aside, "the view drifted back at once");
    let later = frames(&mut world, 90, (Vec2::ZERO, 0.0));
    assert!(later.length() < 0.05, "the view stayed at {later}");
    frames(&mut world, 10, (Vec2::new(0.05, 0.0), 0.0));
    hold(&mut world, true);
    let held = frames(&mut world, 120, (Vec2::ZERO, 0.0));
    assert!(held.x > 0.45, "a held view drifted back to {held}");
}

/// A chase view: with the steer button up the view turns with the nose of the body, frame by
/// frame. With it held the view aims the body and turns with the mouse alone.
#[test]
fn the_view_turns_with_the_body_unless_it_steers() {
    let mut world = gliding();
    frames(&mut world, 1, (Vec2::ZERO, 0.0));
    let carried = frames(&mut world, 30, (Vec2::ZERO, 0.02));
    assert!(
        (carried.x - 0.6).abs() < 1e-4,
        "the view turned by {}",
        carried.x
    );
    hold(&mut world, true);
    let held = frames(&mut world, 30, (Vec2::ZERO, 0.02));
    assert!(
        (held.x - carried.x).abs() < 1e-6,
        "a steering view was carried"
    );
}

/// Off the wings the view is left alone: the mouse turns it and nothing else does.
#[test]
fn off_the_wings_the_view_is_the_mouse_alone() {
    let mut world = gliding();
    let mut bodies = world.query::<&mut Gear>();
    for mut gear in bodies.iter_mut(&mut world) {
        gear.state.mode = Mode::Fall;
    }
    let view = frames(&mut world, 5, (Vec2::new(0.03, -0.02), 0.1));
    assert!(view.distance(Vec2::new(0.15, -0.1)) < 1e-5, "view {view}");
}
