//! The lever and the stick of the assisted ship in space: the lever sets a speed that grows
//! with the clearance, and a released stick stops the turn.

use bevy::math::{DQuat, DVec3};

use super::cruise::{FAR, ORBIT, beside, fly};
use crate::ship::assist::TURN_RATE;
use crate::ship::cruise::top_speed;
use crate::ship::flight::{Controls, STEP};

/// Returns the speed along the nose that a ship at rest reaches in `seconds` with the lever
/// at `throttle`, and the fastest it flew on the way.
fn run_up(gap: f64, throttle: f64, boost: bool, seconds: f64) -> (f64, f64) {
    let mut ship = beside(DVec3::ZERO, throttle);
    let controls = Controls {
        boost,
        ..Controls::default()
    };
    let mut fastest = 0.0f64;
    fly(&mut ship, &controls, (gap, seconds), |_, drift| {
        fastest = fastest.max(drift.length())
    });
    ((ship.vel - ORBIT).dot(ship.forward()), fastest)
}

/// The lever sets a speed: 40 m/s at a surface, 0.625 m/s more with every metre of clearance
/// (0.25 before 0.11.0, when the top speed was half), 30 km/s in open space from 48 km (2,500
/// m/s from 3.9 km before the one speed rule) and twice that with the boost. The ship reaches
/// it within 4.5 seconds and never flies faster than asked.
#[test]
fn the_lever_sets_a_speed_that_grows_with_the_clearance() {
    let tops = [0.0, 100.0, 1000.0, 4000.0, 9840.0, 1e9].map(top_speed);
    assert_eq!((tops[0], tops[5]), (40.0, 30_000.0));
    assert!(tops.windows(2).all(|pair| pair[0] <= pair[1]), "{tops:?}");
    assert_eq!((tops[1], tops[2], tops[3]), (102.5, 665.0, 2540.0));
    for (throttle, boost, asked) in [
        (1.0, false, 30_000.0),
        (0.5, false, 15_000.0),
        (1.0, true, 60_000.0),
    ] {
        let (speed, fastest) = run_up(FAR, throttle, boost, 4.5);
        assert!((speed / asked - 1.0).abs() < 1e-3, "{speed} for {asked}");
        assert!(fastest < asked * 1.001, "flew at {fastest} for {asked}");
    }
    let (slow, _) = run_up(0.0, 1.0, false, 2.0);
    assert!((slow - 40.0).abs() < 1.0, "{slow} m/s at the surface");
}

/// Returns the spin of a ship after `held` seconds of full stick around `axis` and the spins
/// of the 0.3 seconds after the stick is released, with the lever at `throttle`.
fn turn(axis: DVec3, throttle: f64) -> (DVec3, Vec<DVec3>, DQuat) {
    let mut ship = beside(DVec3::ZERO, throttle);
    let stick = Controls {
        turn: axis,
        ..Controls::default()
    };
    fly(&mut ship, &stick, (FAR, 1.5), |_, _| {});
    let (rate, mut after) = (ship.spin, Vec::new());
    for _ in 0..36 {
        fly(&mut ship, &Controls::default(), (FAR, STEP), |_, _| {});
        after.push(ship.spin);
    }
    (rate, after, ship.rot)
}

/// A full stick turns the ship at its turn rate and a released stick stops it within a
/// quarter of a second, without swinging back and without a wobble left. The turn is the same
/// at rest and at full speed.
#[test]
fn a_released_stick_stops_the_turn() {
    for axis in [DVec3::X, DVec3::Y, DVec3::Z, DVec3::new(-1.0, 1.0, -1.0)] {
        let (rate, after, rot) = turn(axis, 0.0);
        assert!((rate - axis * TURN_RATE).length() < 1e-9, "{rate}");
        let slowing = after
            .windows(2)
            .all(|pair| pair[1].length() <= pair[0].length());
        let same_way = after.iter().all(|spin| (*spin * axis).min_element() >= 0.0);
        assert!(slowing && same_way, "{axis}: the turn swung back");
        assert_eq!(after[29], DVec3::ZERO, "{axis}: still turning after 0.25 s");
        assert_eq!(
            turn(axis, 1.0).2,
            rot,
            "{axis}: the turn depends on the speed"
        );
    }
}
