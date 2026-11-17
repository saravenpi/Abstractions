//! Boots alone: a fall brake that can never gain height.

use bevy::prelude::*;

use super::{DT, Flight, braking, looking};
use crate::gear::{GearInput, Mode, descent_target};

/// Vertical speeds the boots are switched on at.
const CLIMBS: [f32; 8] = [-80.0, -40.0, -10.0, -5.0, -2.0, 0.0, 3.0, 7.0];

/// Gravities of the weakest, the home and the strongest planet.
const GRAVITIES: [f32; 3] = [6.6, 22.0, 39.6];

/// Returns the highest point a body reaches in 60 s of boots from `velocity`, sticks pushed.
fn highest(velocity: Vec3, gravity: f32, density: f32) -> f64 {
    let mut flight = Flight::new(Mode::Off, velocity);
    let input = GearInput {
        gravity,
        density,
        pitch: -1.0,
        roll: 1.0,
        look: looking(0.4, 1.2),
        ..braking()
    };
    let mut top = 0.0_f64;
    for _ in 0..3600 {
        flight.tick(&input, DT);
        top = top.max(flight.pos.y);
    }
    top
}

/// Checks that boots switched on at vertical speed `climb` under `gravity` never lift the body.
fn never_rises(climb: f32, gravity: f32) {
    let jump = f64::from(climb.max(0.0).powi(2) / (2.0 * gravity));
    for (drift, density) in [
        (0.0, 1.2),
        (10.0, 1.2),
        (40.0, 1.2),
        (0.0, 0.3),
        (40.0, 0.3),
    ] {
        let top = highest(Vec3::new(drift, climb, 0.0), gravity, density);
        assert!(top <= jump + 1e-3, "g {gravity} climb {climb}: rose {top}");
    }
}

/// From many starts, the body never rises above where the boots came on, beyond the rise its
/// own jump already had.
#[test]
fn boots_never_gain_height() {
    for gravity in GRAVITIES {
        for climb in CLIMBS {
            never_rises(climb, gravity);
        }
    }
}

/// Returns the vertical speed of every frame of 20 s of boots from a vertical speed.
fn descent(climb: f32, gravity: f32) -> Vec<f32> {
    let mut flight = Flight::new(Mode::BootFall, Vec3::Y * climb);
    let input = GearInput {
        gravity,
        ..braking()
    };
    (0..1200)
        .map(|_| {
            flight.tick(&input, DT);
            flight.state.velocity.y
        })
        .collect()
}

/// A fast fall eases up to the target and a slow one eases down to it: always one way, never
/// past it and back.
#[test]
fn boot_descent_converges_without_overshoot() {
    for gravity in GRAVITIES {
        let target = descent_target(gravity);
        assert!((4.0..=6.0).contains(&target));
        let fast = descent(-60.0, gravity);
        let slow = descent(0.0, gravity);
        assert!(
            fast.windows(2).all(|w| w[1] >= w[0] - 1e-5),
            "fast fall reversed"
        );
        assert!(
            slow.windows(2).all(|w| w[1] <= w[0] + 1e-5),
            "slow fall reversed"
        );
        for run in [&fast, &slow] {
            let last = -run[run.len() - 1];
            assert!((last - target).abs() < 0.02 * target, "settled at {last}");
        }
    }
}

/// The boots take about a second to catch a fast fall and give a modest drift, no more.
#[test]
fn boots_brake_soon_and_drift_modestly() {
    let fall = descent(-40.0, 22.0);
    assert!(fall[90] > -8.0, "still falling at {} after 1.5 s", fall[90]);
    let mut flight = Flight::new(Mode::BootFall, Vec3::Y * -5.0);
    let input = GearInput {
        pitch: -1.0,
        ..braking()
    };
    flight.run(&input, 20.0);
    let drift = flight.state.velocity.with_y(0.0).length();
    assert!((6.0..=12.0).contains(&drift), "drift {drift}");
    assert!(
        flight.state.velocity.z < 0.0,
        "forward stick must drift forward"
    );
}
