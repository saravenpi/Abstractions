//! The sense of the keys: every key means the same thing in the frame of the body, whatever
//! its attitude, its heading and the place.
//!
//! The rule of the game: the keys are relative to the body and never to the camera. The roll
//! keys roll it round its own nose, the yaw keys turn its path toward its own wing, and the
//! pitch keys pull toward its own back, with the camera behind, beside or in front. Only while
//! the steer button is held does the view count, and then the body flies to where it looks.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;

use super::Flight;
use super::pilot::keys;
use crate::gear::aim::wrap;
use crate::gear::{FlightState, GearInput, Mode};

/// Seconds a key is held in the table.
const HELD: f32 = 0.3;

/// Airspeed of the bodies of the table in metres per second, where the keys have their full grip.
const SPEED: f32 = 45.0;

/// Least share of a quarter turn the wings must have rolled after [`HELD`], as a sine.
const ROLLED: f32 = 0.3;

/// Least speed in metres per second the yaw key must have put across the path after [`HELD`].
const TURNED: f32 = 2.0;

/// The attitudes of the table as a name, a pitch and a bank, the ones on the back first: after a
/// half roll, in a turn on the back, on the top of a loop, levelled out of a half loop, then on
/// edge both ways, level, banked, and with the nose near straight up and straight down.
const ATTITUDES: [(&str, f32, f32); 11] = [
    ("on its back", 0.0, PI),
    ("on its back in a turn", 0.0, 2.6),
    ("on the top of a loop", PI, 0.0),
    ("levelled out of a half loop", PI, PI),
    ("on its right edge", 0.0, FRAC_PI_2),
    ("on its left edge", 0.0, -FRAC_PI_2),
    ("level", 0.0, 0.0),
    ("banked", 0.0, 0.7),
    ("climbing", 1.4, 0.0),
    ("diving", -1.4, 0.0),
    ("on its back past the vertical", 1.9, 0.0),
];

/// The local ups of the table: the middle of each face of the cube of the planet, a place a
/// degree from a pole and one at a middle latitude.
const UPS: [Vec3; 8] = [
    Vec3::X,
    Vec3::NEG_X,
    Vec3::Y,
    Vec3::NEG_Y,
    Vec3::Z,
    Vec3::NEG_Z,
    Vec3::new(0.017_452, 0.999_848, 0.0),
    Vec3::new(0.5, 0.5, -0.707_107),
];

/// Returns a body on its wings with its nose at `heading`, pitched and banked as `attitude`
/// says, flying along its nose at [`SPEED`].
pub(super) fn posed(heading: f32, attitude: (f32, f32)) -> Flight {
    let mut flight = Flight::new(Mode::Glide, Vec3::ZERO);
    flight.state.pitch = attitude.0;
    flight.state.bank = attitude.1;
    flight.state.servo.yaw = heading;
    flight.state.servo.climb = attitude.0;
    flight.state.velocity = flight.state.nose() * SPEED;
    flight.state.airspeed = SPEED;
    flight
}

/// Returns `flight` after `input` was held for [`HELD`].
fn after(flight: &Flight, input: &GearInput) -> FlightState {
    let mut flown = *flight;
    flown.run(input, HELD);
    flown.state
}

/// Returns how far a roll key at `roll` rolled `flight` round its nose toward its right wing,
/// as the sine of the angle, seen in the axes `place` turns the upright frame to: the lesser of
/// how far the back went to where the right wing was and the right wing to where the belly was.
fn rolled(flight: &Flight, place: Quat, roll: f32) -> f32 {
    let (from, to) = (
        flight.state.frame(),
        after(flight, &keys(0.0, roll)).frame(),
    );
    let over = (place * to * Vec3::Y).dot(place * from * Vec3::X);
    let under = (place * to * Vec3::X).dot(place * from * Vec3::NEG_Y);
    over.min(under) * roll.signum()
}

/// Returns the speed a yaw key at `yaw` put on `flight` toward its right wing, in metres per
/// second, seen in the axes `place` turns the upright frame to: what the key changed against the
/// same flight with no key held.
fn turned(flight: &Flight, place: Quat, yaw: f32) -> f32 {
    let free = after(flight, &keys(0.0, 0.0));
    let held = after(
        flight,
        &GearInput {
            yaw,
            ..keys(0.0, 0.0)
        },
    );
    let wing = place * flight.state.frame() * Vec3::X;
    (place * (held.velocity - free.velocity)).dot(wing) * yaw.signum()
}

/// Every 30 degrees round the compass, at every place and in every attitude of the table, the
/// right roll key rolls the body round its own nose with its right wing going toward its belly
/// and the left key the other way, and the right yaw key turns the flight path toward the right
/// wing and the left key toward the left. The sense never flips.
///
/// A yaw turn round the local up, which the gear made until this test, fails here: on its back
/// it puts about 5 m/s toward the wrong wing, on edge none and with the nose up about 1 m/s.
#[test]
fn the_keys_keep_their_sense_in_the_frame_of_the_body() {
    for (name, pitch, bank) in ATTITUDES {
        for step in 0..12 {
            let heading = wrap(step as f32 * TAU / 12.0);
            let flight = posed(heading, (pitch, bank));
            for (up, key) in UPS.iter().flat_map(|up| [(*up, 1.0), (*up, -1.0)]) {
                let place = Quat::from_rotation_arc(Vec3::Y, up.normalize());
                let at = format!("{name}, heading {heading}, up {up}, key {key}");
                let roll = rolled(&flight, place, key);
                assert!(roll > ROLLED, "{at}: rolled {roll}");
                let turn = turned(&flight, place, key);
                assert!(turn > TURNED, "{at}: turned {turn} m/s");
            }
        }
    }
}

/// The camera in front of the body changes nothing: with the steer button up the keys never read
/// the view, so the same keys fly the same flight with the view behind, beside, in front, above
/// or on its back. Body-relative control is the rule, as in a plane seen from outside: seen from
/// the front, the right roll key rolls the body to the left of the screen.
#[test]
fn the_keys_do_not_read_where_the_camera_is() {
    let flight = posed(0.6, (0.1, 0.4));
    let held = GearInput {
        yaw: 1.0,
        ..keys(0.5, 1.0)
    };
    let behind = after(&flight, &held);
    for look in [Vec3::X, Vec3::Z, Vec3::Y, Vec3::new(0.6, -0.5, 0.6)] {
        let seen = after(&flight, &GearInput { look, ..held });
        assert_eq!(seen, behind, "the view along {look} changed the flight");
    }
}
