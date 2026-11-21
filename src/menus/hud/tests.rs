//! HUD tests: unit formatting at each boundary, what each instrument shows and the banner.

use super::banner::opacity;
use super::flight::{Flight, Gauge, reading};
use super::units::{height, percent, speed};
use crate::ship::ShipTelemetry;
use crate::space::Frame;
use crate::universe::{BodyId, StarId};
use bevy::math::Vec3;

/// Heights switch unit at 1000 of the smaller one, keep three digits and keep their sign.
#[test]
fn heights_cover_each_unit_and_sign() {
    let cases = [
        (0.0, "0 m"),
        (-0.4, "-0.4 m"),
        (12.4, "12.4 m"),
        (999.4, "999 m"),
        (999.5, "1 km"),
        (-999.5, "-1 km"),
        (12_345.0, "12.3 km"),
        (999_499.0, "999 km"),
        (999_500.0, "1 000 km"),
        (-4.0e7, "-40 000 km"),
        (9.994e8, "999 000 km"),
        (9.995e8, "1 million km"),
        (1.5e13, "100 AU"),
    ];
    for (metres, shown) in cases {
        assert_eq!(height(metres), shown, "{metres}");
    }
}

/// Speeds use the same steps with a per-second unit, and levers read as whole percentages.
#[test]
fn speeds_and_levers_format_at_each_boundary() {
    let cases = [
        (0.0, "0 m/s"),
        (999.4, "999 m/s"),
        (999.5, "1 km/s"),
        (7_800.0, "7.8 km/s"),
        (999_500.0, "1 000 km/s"),
        (9.995e8, "3.33 x light speed"),
        (3.0e10, "100 x light speed"),
    ];
    for (value, shown) in cases {
        assert_eq!(speed(value), shown, "{value}");
    }
    assert_eq!(speed(-12.0), "12 m/s");
    let levers = [
        (0.0, "0 %"),
        (0.654, "65 %"),
        (1.0, "100 %"),
        (-0.5, "-50 %"),
    ];
    for (value, shown) in levers {
        assert_eq!(percent(value), shown, "{value}");
    }
}

/// Returns the instruments of a ship flying in `frame`.
fn flying(frame: Frame) -> Flight {
    Flight {
        telemetry: ShipTelemetry {
            altitude_ground: 1240.0,
            surface_speed: 312.0,
            orbital_speed: 7_800.0,
            throttle: 0.65,
            landed: false,
            frame,
            apoapsis: Some(52_000.0),
            periapsis: Some(-1_200.0),
            ..ShipTelemetry::default()
        },
        ..Flight::default()
    }
}

/// In the air the speed is the surface speed and no orbit shows; in space the orbit shows. A
/// landed ship under the assist says that the rise key takes it off.
#[test]
fn instruments_follow_the_frame() {
    let air = flying(Frame::Surface(BodyId::default()));
    assert_eq!(reading(Gauge::Altitude, &air).unwrap(), "1.24 km");
    assert_eq!(reading(Gauge::Speed, &air).unwrap(), "312 m/s surface");
    assert_eq!(reading(Gauge::Throttle, &air).unwrap(), "65 %");
    assert_eq!(reading(Gauge::Assist, &air).unwrap(), "on");
    assert_eq!(reading(Gauge::Apoapsis, &air), None);
    let mut landed = flying(Frame::Surface(BodyId::default()));
    landed.telemetry.landed = true;
    assert_eq!(
        reading(Gauge::Assist, &landed).unwrap(),
        "on - rise to take off"
    );
    let space = flying(Frame::System(StarId::default()));
    assert_eq!(reading(Gauge::Speed, &space).unwrap(), "7.8 km/s orbital");
    assert_eq!(reading(Gauge::Apoapsis, &space).unwrap(), "52 km");
    assert_eq!(reading(Gauge::Periapsis, &space).unwrap(), "-1.2 km");
}

/// The banner fades in, holds at full opacity and fades out to nothing.
#[test]
fn the_banner_fades_in_and_out() {
    assert_eq!(opacity(4.5), 0.0);
    assert_eq!(opacity(2.0), 1.0);
    assert_eq!(opacity(0.0), 0.0);
    assert!(opacity(4.3) > 0.0 && opacity(4.3) < 1.0);
    assert!(opacity(0.2) > 0.0 && opacity(0.2) < 1.0);
}

/// A target to the right reads a quarter turn to the right and sits right of the middle, one
/// straight ahead sits in the middle, and one behind lies past the end of its side.
#[test]
fn the_compass_points_at_its_target() {
    use super::needle::{SPAN, bearing, slot};
    let (ahead, up) = (Vec3::NEG_Z, Vec3::Y);
    let right = bearing(Vec3::new(30.0, 4.0, 0.0), ahead, up).unwrap();
    assert!((right - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    assert_eq!(slot(bearing(Vec3::NEG_Z * 9.0, ahead, up).unwrap()), 0.5);
    assert!((slot(SPAN * 0.5) - 0.75).abs() < 1e-6);
    let behind = bearing(Vec3::new(-1.0, 0.0, 9.0), ahead, up).unwrap();
    assert_eq!((slot(behind), slot(right)), (0.0, 1.0));
    assert_eq!(bearing(Vec3::Y, ahead, up), None);
}
