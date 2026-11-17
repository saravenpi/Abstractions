//! Meeting the ground and walls: slide into a run, roll out a steep landing, glance off a cliff.

use bevy::prelude::*;

use super::transitions::TWIST;
use super::{DT, Flight, flying, gliding};
use crate::gear::{GearInput, Mode, deflect, slid};

/// Lands `flight` on flat ground: the test plays the controller, which finds the ground, moves
/// the body and hands back a velocity with nothing left going into the surface.
fn land(flight: &mut Flight, input: &GearInput, seconds: f32) {
    let grounded = GearInput {
        grounded: true,
        ..*input
    };
    for _ in 0..(seconds / DT).round() as u32 {
        flight.tick(&grounded, DT);
        let moved = flight
            .state
            .velocity
            .with_y(flight.state.velocity.y.max(0.0));
        flight.state = slid(&flight.state, moved);
    }
}

/// A shallow touchdown at glide speed slides: the momentum carries on, the speed bleeds off
/// without a stop, and the slide hands over to the run at running speed.
#[test]
fn shallow_touchdown_slides_into_a_run() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -6.0, -40.0));
    flight.run(&gliding(-0.1), 0.5);
    let arriving = flight.state.velocity.with_y(0.0).length();
    land(&mut flight, &gliding(-0.1), DT);
    assert_eq!(flight.state.mode, Mode::Slide);
    assert!(
        flight.state.velocity.length() > 0.97 * arriving,
        "lost the momentum"
    );
    land(&mut flight, &gliding(-0.1), 0.5);
    assert!(
        flight.state.velocity.length() > 0.4 * arriving,
        "stopped dead"
    );
    land(&mut flight, &gliding(-0.1), 5.0);
    assert_eq!(flight.state.mode, Mode::Off);
    let handed = flight.state.velocity.length();
    assert!((6.0..=8.1).contains(&handed), "handed over at {handed}");
    assert!(
        flight.kick < 0.05 * arriving,
        "velocity jumped by {}",
        flight.kick
    );
    assert!(flight.twist < TWIST, "attitude jumped by {}", flight.twist);
    assert_eq!(flight.state.wings, 0.0);
}

/// A steep fast impact becomes a landing roll: the gear itself never jumps the velocity, it
/// sheds most of the speed within the roll and ends standing.
#[test]
fn steep_impact_rolls_out() {
    let mut flight = Flight::new(Mode::Fall, Vec3::new(0.0, -45.0, -30.0));
    let arriving = flight.state.velocity.length();
    land(&mut flight, &GearInput::default(), DT);
    assert_eq!(flight.state.mode, Mode::Roll);
    assert!(
        flight.kick < 0.09 * arriving,
        "velocity jumped by {}",
        flight.kick
    );
    land(&mut flight, &GearInput::default(), 2.0);
    assert_eq!(flight.state.mode, Mode::Off);
    assert!(flight.state.velocity.length() < 8.1);
    assert!(
        flight.kick < 0.09 * arriving,
        "velocity jumped by {}",
        flight.kick
    );
    assert!(flight.twist < TWIST, "attitude jumped by {}", flight.twist);
}

/// A slow landing under the boots simply stands, and a hit the physics already absorbed still
/// counts as steep.
#[test]
fn slow_landings_stand_and_absorbed_hits_roll() {
    let mut soft = Flight::new(Mode::BootFall, Vec3::new(2.0, -5.0, 0.0));
    land(&mut soft, &super::braking(), DT);
    assert_eq!(soft.state.mode, Mode::Off);
    let mut hard = Flight::new(Mode::Fall, Vec3::new(0.0, -50.0, -20.0));
    hard.state = slid(&hard.state, Vec3::new(0.0, 0.0, -20.0));
    land(&mut hard, &GearInput::default(), DT);
    assert_eq!(hard.state.mode, Mode::Roll);
}

/// Leaving the ground mid-slide with the gear held goes straight back to flying.
#[test]
fn sliding_off_an_edge_flies_again() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, -3.0, -50.0));
    land(&mut flight, &flying(0.0), 0.3);
    assert_eq!(flight.state.mode, Mode::Slide);
    flight.run(&flying(0.0), 1.0);
    assert_eq!(flight.state.mode, Mode::Fly);
    assert!(flight.twist < TWIST, "attitude jumped by {}", flight.twist);
}

/// A wall hit slides along the wall and costs speed in proportion to how square it was.
#[test]
fn wall_hits_cost_speed_by_impact_angle() {
    let speed = 50.0_f32;
    let mut kept = [0.0_f32; 4];
    for (slot, degrees) in [5.0_f32, 20.0, 45.0, 80.0].into_iter().enumerate() {
        let angle = degrees.to_radians();
        let before = Vec3::new(angle.sin(), 0.0, -angle.cos()) * speed;
        let after = deflect(before, before.with_x(0.0));
        assert!(after.x == 0.0 && after.z < 0.0, "must slide along the wall");
        kept[slot] = after.length() / speed;
    }
    assert!(kept[0] > 0.96, "a glancing touch cost {}", 1.0 - kept[0]);
    assert!(
        kept.windows(2).all(|w| w[1] < w[0]),
        "loss must grow with the angle"
    );
    assert!(kept[3] < 0.15);
    assert_eq!(deflect(Vec3::X * speed, Vec3::X * speed), Vec3::X * speed);
}
