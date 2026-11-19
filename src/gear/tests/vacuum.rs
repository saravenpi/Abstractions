//! Thin air and none: forces fade with density and vanish in vacuum.

use bevy::prelude::*;

use super::{Flight, braking, flying, gliding};
use crate::gear::{FlightState, Gear, GearInput, Mode, air_density};

/// In zero density every mode is a plain ballistic fall: no lift, no drag, no booster push.
#[test]
fn vacuum_leaves_only_gravity() {
    let inputs = [braking(), gliding(0.4), flying(0.6), GearInput::default()];
    for (mode, input) in [Mode::BootFall, Mode::Glide, Mode::Fly, Mode::Fall]
        .into_iter()
        .zip(inputs)
    {
        let start = Vec3::new(12.0, -30.0, -55.0);
        let mut flight = Flight::new(mode, start);
        let airless = GearInput {
            density: 0.0,
            pitch: 1.0,
            roll: -1.0,
            aim: Some(Vec2::new(0.8, 0.5)),
            ..input
        };
        flight.run(&airless, 10.0);
        let ballistic = start - Vec3::Y * 220.0;
        assert!(
            (flight.state.velocity - ballistic).length() < 0.05,
            "{mode:?}"
        );
        assert!(
            flight.state.load.abs() < 1e-3,
            "{mode:?} felt {}",
            flight.state.load
        );
    }
}

/// The air thins with height, so level flight has a ceiling below the top of the atmosphere.
#[test]
fn thin_air_sets_a_ceiling() {
    let top = 6000.0;
    assert_eq!(air_density(top, 0.0), 1.2);
    assert_eq!(air_density(top, top), 0.0);
    assert!(air_density(top, 750.0) < air_density(top, 0.0) * 0.4);
    let holds = |altitude: f32| {
        let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -70.0));
        let input = GearInput {
            density: air_density(top, altitude),
            ..flying(0.0)
        };
        flight.run(&input, 60.0);
        flight.pos.y > -20.0
    };
    assert!(holds(0.0) && holds(600.0), "must fly in thick air");
    assert!(!holds(0.6 * top), "must not hold height near the top");
}

/// Level flight holds its height just under the documented ceiling and not just above it.
#[test]
fn the_ceiling_is_the_documented_share_of_the_atmosphere() {
    let kept = |share: f32| {
        let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -70.0));
        let input = GearInput {
            density: air_density(1.0, share),
            ..flying(0.05)
        };
        flight.run(&input, 60.0);
        flight.pos.y
    };
    let share = crate::gear::tune::CEILING_SHARE;
    assert!(
        kept(share - 0.05) > 0.0,
        "sank {} below the ceiling",
        kept(share - 0.05)
    );
    assert!(kept(share + 0.07) < 0.0, "climbed above the ceiling");
}

/// Returns nothing: it only compiles for types that own no heap memory.
const fn plain<T: Copy>() {}

/// Nothing in a step can allocate: state, input and component are plain copyable data of a
/// fixed small size, with no `Vec` or `Box` anywhere.
#[test]
fn step_cannot_allocate() {
    plain::<FlightState>();
    plain::<GearInput>();
    plain::<Gear>();
    assert!(
        size_of::<FlightState>() <= 112,
        "state grew to {}",
        size_of::<FlightState>()
    );
}
