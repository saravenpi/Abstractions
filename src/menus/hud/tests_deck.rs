//! HUD tests: the flight instruments over a gas giant and on a moon, and the landing gear.

use super::flight::{Flight, Gauge, reading};
use crate::ship::ShipTelemetry;
use crate::space::Frame;
use crate::universe::{BodyId, StarId};

/// Returns the instruments of a ship flying 1.24 km over the ground and 3.40 km over the sea
/// level of `frame`.
fn flying(frame: Frame) -> Flight {
    Flight {
        telemetry: ShipTelemetry {
            altitude_ground: 1240.0,
            altitude_sea: 3400.0,
            landed: false,
            frame,
            ..ShipTelemetry::default()
        },
        ..Flight::default()
    }
}

/// Over a gas giant the altitude is the height above its cloud deck and the frame says the
/// ship cannot land; a moon is a surface like any planet, and the destination names it.
#[test]
fn a_gas_giant_has_a_deck_and_a_moon_a_surface() {
    let moon = BodyId {
        moon: 1,
        ..BodyId::default()
    };
    let mut deep = flying(Frame::Surface(moon));
    assert_eq!(reading(Gauge::Altitude, &deep).unwrap(), "1.24 km");
    assert_eq!(reading(Gauge::Frame, &deep).unwrap(), "surface");
    deep.destination = Some(("Ola".to_string(), 2.0e5));
    assert_eq!(reading(Gauge::Destination, &deep).unwrap(), "Ola  200 km");
    deep.giant = true;
    assert_eq!(
        reading(Gauge::Altitude, &deep).unwrap(),
        "3.40 km above deck"
    );
    let frame = reading(Gauge::Frame, &deep).unwrap();
    assert_eq!(frame, "surface - no ground, cannot land");
    let space = Flight {
        giant: true,
        ..flying(Frame::System(StarId::default()))
    };
    assert_eq!(reading(Gauge::Frame, &space).unwrap(), "orbit");
}

/// The gear instrument reads up, moving and down near a planet with ground, and shows nothing
/// in space, over a gas giant or without a ship.
#[test]
fn the_gear_reads_up_moving_and_down_near_a_planet() {
    let mut near = flying(Frame::Surface(BodyId::default()));
    assert_eq!(reading(Gauge::Gear, &near), None);
    for (gear, shown) in [
        (0.0, "up"),
        (0.01, "moving"),
        (0.99, "moving"),
        (1.0, "down"),
    ] {
        near.gear = Some(gear);
        assert_eq!(reading(Gauge::Gear, &near).unwrap(), shown, "{gear}");
    }
    let giant = Flight {
        giant: true,
        ..near.clone()
    };
    assert_eq!(reading(Gauge::Gear, &giant), None);
    let space = Flight {
        gear: Some(1.0),
        ..flying(Frame::System(StarId::default()))
    };
    assert_eq!(reading(Gauge::Gear, &space), None);
    assert!(Gauge::ALL.contains(&(Gauge::Gear, "GEAR")));
}
