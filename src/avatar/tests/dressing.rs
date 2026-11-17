//! Dressing the pose from the flight gear and the view.

use bevy::prelude::*;

use crate::avatar::dress::{dress, mode, shown};
use crate::avatar::{AvatarPose, Mode, Shown};
use crate::gear::{FlightState, Mode as Flight};

/// Returns a gear in `mode` moving at `velocity`, wings and boots fully out.
fn gear(mode: Flight, velocity: Vec3) -> FlightState {
    FlightState {
        mode,
        velocity,
        wings: 1.0,
        thrust: 1.0,
        bank: 0.4,
        pitch: -0.2,
        airspeed: velocity.length(),
        ..FlightState::default()
    }
}

/// Each gear mode poses its own way, a deep swim dives, and the filler decides with the gear off.
#[test]
fn the_gear_chooses_the_flight_poses() {
    let (still, up, down) = (Vec3::ZERO, Vec3::Y * 3.0, Vec3::NEG_Y);
    let cases = [
        (Mode::Fall, Flight::BootFall, still, false, Mode::BootFall),
        (Mode::Fall, Flight::Glide, still, false, Mode::Glide),
        (Mode::Fall, Flight::Fly, still, false, Mode::Fly),
        (Mode::Fall, Flight::Fall, up, false, Mode::Jump),
        (Mode::Jump, Flight::Fall, down, false, Mode::Fall),
        (Mode::Run, Flight::Roll, still, false, Mode::Land),
        (Mode::Run, Flight::Slide, still, false, Mode::Run),
        (Mode::Swim, Flight::Off, still, true, Mode::Dive),
        (Mode::Swim, Flight::Off, still, false, Mode::Swim),
        (Mode::Walk, Flight::Off, still, true, Mode::Walk),
    ];
    for (filled, flight, velocity, deep, wanted) in cases {
        assert_eq!(
            mode(filled, &gear(flight, velocity), deep),
            wanted,
            "{flight:?}"
        );
    }
}

/// In the air the bank, the pitch, the push and the wings reach the pose; on the ground the
/// trunk stands level whatever the gear last held.
#[test]
fn the_flight_poses_the_body_only_in_the_air() {
    let mut pose = AvatarPose::default();
    dress(
        &mut pose,
        &gear(Flight::Glide, Vec3::NEG_Z * 30.0),
        Vec2::new(0.3, -0.1),
        false,
    );
    assert_eq!((pose.bank, pose.pitch, pose.airspeed), (0.4, -0.2, 30.0));
    assert!(pose.wings > 0.99 && pose.thrust > 0.99 && pose.look == Vec2::new(0.3, -0.1));
    dress(
        &mut pose,
        &gear(Flight::Slide, Vec3::NEG_Z * 9.0),
        Vec2::ZERO,
        false,
    );
    assert_eq!((pose.bank, pose.pitch), (0.0, 0.0));
    let mut rolled = AvatarPose::default();
    dress(
        &mut rolled,
        &gear(Flight::Roll, Vec3::NEG_Z),
        Vec2::ZERO,
        false,
    );
    assert!(rolled.landed_hard >= 0.7);
}

/// Seated the body is hidden; at the eyes only the head is; behind it all of it is drawn.
#[test]
fn the_camera_chooses_what_is_drawn() {
    assert_eq!(shown(true, 1.0), Shown::Hidden);
    assert_eq!(shown(false, 0.0), Shown::FirstPerson);
    assert_eq!(shown(false, 0.3), Shown::FirstPerson);
    assert_eq!(shown(false, 1.0), Shown::Full);
}
