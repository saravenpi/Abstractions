//! Tests of the instrument rows: the warp and glide speeds, and the rows of an orbit.

use super::context::{Context, shown};
use super::flight::{Flight, Gauge, reading};
use crate::ship::{ShipTelemetry, WarpState};
use crate::space::Frame;
use crate::universe::StarId;

/// Returns the instruments of a ship flying in the space of a star.
fn flying() -> Flight {
    Flight {
        telemetry: ShipTelemetry {
            altitude_ground: 1240.0,
            orbital_speed: 7_800.0,
            throttle: 0.65,
            landed: false,
            frame: Frame::System(StarId::default()),
            apoapsis: Some(52_000.0),
            periapsis: Some(-1_200.0),
            ..ShipTelemetry::default()
        },
        ..Flight::default()
    }
}

/// The warp speed replaces the speed in warp, and a gliding body reads its own height and air
/// speed, not the ship's.
#[test]
fn warp_and_glide_speeds_replace_the_ship_speed() {
    let mut flight = flying();
    flight.telemetry.warp = WarpState::Active(2.0e10);
    assert_eq!(reading(Gauge::Speed, &flight).unwrap(), "2.00 gly/s warp");
    let gliding = Flight {
        glide: Some((812.0, 41.0)),
        ..Flight::default()
    };
    assert_eq!(reading(Gauge::Altitude, &gliding).unwrap(), "812 m");
    assert_eq!(reading(Gauge::Speed, &gliding).unwrap(), "41 m/s air");
}

/// The rows the context shows for a ship in orbit are exactly altitude, speed, throttle,
/// stabiliser, highest and lowest point, in that order.
#[test]
fn the_orbit_rows_are_the_spec_rows() {
    let flight = flying();
    let rows: Vec<&str> = Gauge::ALL
        .into_iter()
        .filter(|(gauge, _)| shown(Context::ShipOrbit).contains(gauge.element()))
        .filter(|(gauge, _)| reading(*gauge, &flight).is_some())
        .map(|(_, name)| name)
        .collect();
    assert_eq!(
        rows,
        [
            "Altitude",
            "Speed",
            "Throttle",
            "Stabiliser",
            "Highest",
            "Lowest"
        ]
    );
    assert_eq!(Gauge::ALL.len(), 7);
}
