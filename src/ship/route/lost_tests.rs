//! The lost reading of a dropped route clears on every path the pilot has: Go again, another target, stopping,
//! the lever or the stick, landing, and time.

use bevy::math::{DQuat, DVec3};

use super::steer::{LOST_SHOWN, recover};
use crate::computer::{Block, Press, RouteStatus, Target, press};
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::helm::Helm;
use crate::ship::pilot::Autopilot;
use crate::space::Frame;
use crate::universe::{BodyId, StarId};

/// A route that was dropped for coming no nearer.
fn lost() -> Autopilot {
    Autopilot {
        lost: true,
        ..Autopilot::default()
    }
}

/// A ship in space with the engines idle.
fn idle() -> Ship {
    Ship::at_rest(
        Frame::System(StarId::default()),
        DVec3::ZERO,
        DQuat::IDENTITY,
    )
}

/// Runs one frame of `dt` seconds of the recovery and returns whether the route still reads as lost.
fn reads_lost(mut pilot: Autopilot, ship: &Ship, helm: &Helm, dt: f32) -> bool {
    recover(&mut pilot, (ship, helm), dt);
    pilot.lost
}

/// An idle ship in space keeps the reading for a moment, then lets it go after [`LOST_SHOWN`] seconds.
#[test]
fn the_reading_lasts_a_few_seconds() {
    let (ship, helm) = (idle(), Helm::default());
    assert!(reads_lost(lost(), &ship, &helm, 1.0));
    let mut pilot = lost();
    for _ in 0..8 {
        recover(&mut pilot, (&ship, &helm), 1.0);
    }
    assert!(pilot.lost);
    recover(&mut pilot, (&ship, &helm), 1.0);
    assert!(!pilot.lost);
}

/// The lever, the stick and a landing clear it at once.
#[test]
fn the_pilot_taking_over_clears_it() {
    let helm = Helm::default();
    let mut lever = idle();
    lever.throttle = 0.4;
    assert!(!reads_lost(lost(), &lever, &helm, 0.016));
    let stick = Helm {
        controls: Controls {
            lift: 1.0,
            ..Controls::default()
        },
        ..Helm::default()
    };
    assert!(!reads_lost(lost(), &idle(), &stick, 0.016));
    let mut landed = idle();
    landed.landed = true;
    assert!(!reads_lost(lost(), &landed, &helm, 0.016));
}

/// Stopping the route and engaging another target clear it, and Go on a course that read as lost tries it
/// again, where any other block keeps the key from doing anything.
#[test]
fn go_stop_and_a_new_target_clear_it() {
    let star = Target::Star(StarId::default());
    let mut stopped = lost();
    stopped.stop();
    assert!(!stopped.lost);
    let mut again = lost();
    again.engage(star);
    assert!(!again.lost);
    let status = RouteStatus::Blocked(Block::Lost);
    assert_eq!(press(Some(star), Some(star), status, true), Press::Go(star));
    let stuck = RouteStatus::Blocked(Block::Landed);
    assert_eq!(press(Some(star), Some(star), stuck, true), Press::Nothing);
    let planet = Target::Body(BodyId::default());
    assert_eq!(press(Some(planet), None, status, true), Press::Go(planet));
}
