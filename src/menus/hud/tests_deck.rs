//! HUD tests: the flight instruments over a gas giant and on a moon.

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
