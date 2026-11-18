//! The ground assist: it softens steep arrivals and never takes the stick away.

use bevy::prelude::*;

use super::{DT, Flight, gliding};
use crate::gear::{FlightState, GearInput, Mode};

/// Height of the dive above the ground at its start, in metres.
const HEIGHT: f64 = 150.0;

/// Dives at the ground with the pitch stick at `stick` and returns the state at the touchdown.
///
/// A stick held part way holds a dive: it turns the course down as fast as the course eases
/// back, the steeper the farther it is held. Held all the way it pushes over and round.
fn dive(assist: f32, stick: f32) -> FlightState {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -40.0, -45.0));
    while flight.pos.y > -HEIGHT {
        let input = GearInput {
            assist,
            pitch: stick,
            clearance: (HEIGHT + flight.pos.y) as f32,
            ..gliding(0.0)
        };
        flight.tick(&input, DT);
    }
    flight.state
}

/// With the assist a steep dive arrives flatter and sinking slower than without it.
#[test]
fn assist_flattens_a_steep_arrival() {
    let (helped, alone) = (dive(1.0, -0.4), dive(0.0, -0.4));
    assert!(
        helped.velocity.y > alone.velocity.y + 5.0,
        "{} vs {}",
        helped.velocity.y,
        alone.velocity.y
    );
    assert!(helped.nose().y > alone.nose().y + 0.1);
}

/// The stick still works through the assist: pushing down harder arrives steeper than a light
/// push.
#[test]
fn assist_leaves_the_stick_in_charge() {
    let (pushed, hands_off) = (dive(1.0, -0.8), dive(1.0, -0.2));
    let (down, level) = (pushed.nose().y, hands_off.nose().y);
    assert!(down < level - 0.05, "nose {down} vs {level}");
    assert!(pushed.velocity.y < hands_off.velocity.y - 2.0);
}

/// Level flight close to the ground and unknown clearance get no bias at all.
#[test]
fn assist_is_silent_in_level_flight() {
    let fly = |clearance: f32| {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -2.0, -45.0));
        let input = GearInput {
            clearance,
            ..gliding(-0.05)
        };
        flight.run(&input, 0.5);
        flight.state.pitch
    };
    assert_eq!(fly(3.0), fly(f32::INFINITY));
}
