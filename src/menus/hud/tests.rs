//! HUD tests: unit formatting at each boundary, what each instrument shows, the prompt and the banner.

use super::banner::opacity;
use super::flight::{Flight, Gauge, reading};
use super::prompt::board_prompt;
use super::units::{height, percent, speed};
use crate::computer::RouteStatus;
use crate::ship::{ShipTelemetry, WarpState};
use crate::space::{Frame, Travel};
use crate::universe::{BodyId, StarId};
use bevy::math::Vec3;

/// Heights switch unit at 1000 of the smaller one, keep three digits and keep their sign.
#[test]
fn heights_cover_each_unit_and_sign() {
    let cases = [
        (0.0, "0 m"),
        (-0.4, "0 m"),
        (12.4, "12 m"),
        (999.4, "999 m"),
        (999.5, "1.00 km"),
        (-999.5, "-1.00 km"),
        (12_345.0, "12.3 km"),
        (999_499.0, "999 km"),
        (999_500.0, "1.00 Mm"),
        (-4.0e7, "-40.0 Mm"),
        (9.994e8, "999 Mm"),
        (9.995e8, "0.10 gly"),
        (1.5e13, "1500 gly"),
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
        (999.5, "1.00 km/s"),
        (7_800.0, "7.80 km/s"),
        (999_500.0, "1.00 Mm/s"),
        (9.995e8, "0.10 gly/s"),
        (3.0e10, "3.00 gly/s"),
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
    assert_eq!(reading(Gauge::Frame, &air).unwrap(), "surface");
    assert_eq!(reading(Gauge::Apoapsis, &air), None);
    assert_eq!(reading(Gauge::Warp, &air), None);
    assert_eq!(reading(Gauge::Destination, &air), None);
    let mut landed = flying(Frame::Surface(BodyId::default()));
    landed.telemetry.landed = true;
    assert_eq!(
        reading(Gauge::Assist, &landed).unwrap(),
        "on - rise to take off"
    );
    let space = flying(Frame::System(StarId::default()));
    assert_eq!(reading(Gauge::Speed, &space).unwrap(), "7.80 km/s orbital");
    assert_eq!(reading(Gauge::Frame, &space).unwrap(), "orbit");
    assert_eq!(reading(Gauge::Apoapsis, &space).unwrap(), "52.0 km");
    assert_eq!(reading(Gauge::Periapsis, &space).unwrap(), "-1.20 km");
}

/// The warp line shows the spool progress, then the warp speed replaces the speed, and a set
/// destination shows its name, its distance and the route state.
#[test]
fn warp_and_destination_show_only_when_needed() {
    let mut flight = flying(Frame::System(StarId::default()));
    flight.telemetry.warp = WarpState::Spooling(0.434);
    assert_eq!(reading(Gauge::Warp, &flight).unwrap(), "spooling 43 %");
    flight.telemetry.warp = WarpState::Active(2.0e10);
    assert_eq!(reading(Gauge::Warp, &flight).unwrap(), "engaged");
    assert_eq!(reading(Gauge::Speed, &flight).unwrap(), "2.00 gly/s warp");
    assert_eq!(reading(Gauge::Altitude, &flight), None);
    flight.destination = Some(("Kodya".to_string(), 1.2e6));
    assert_eq!(
        reading(Gauge::Destination, &flight).unwrap(),
        "Kodya  1.20 Mm"
    );
    flight.route = RouteStatus::Spooling;
    assert_eq!(
        reading(Gauge::Destination, &flight).unwrap(),
        "Kodya  1.20 Mm  spooling"
    );
    assert_eq!(Gauge::ALL.len(), 10);
}

/// The prompt shows at the hatch of the landed ship and inside it, and nowhere else.
#[test]
fn the_board_prompt_needs_the_hatch_or_a_landed_ship() {
    assert_eq!(board_prompt(Travel::OnFoot, true, true), Some("Board"));
    assert_eq!(board_prompt(Travel::OnFoot, false, true), None);
    assert_eq!(
        board_prompt(Travel::InShip, false, true),
        Some("Leave the ship")
    );
    assert_eq!(board_prompt(Travel::InShip, true, false), None);
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
/// straight ahead sits in the middle, and one behind waits at the end of its side.
#[test]
fn the_compass_points_at_its_target() {
    use super::needle::{SPAN, bearing, beside, slot};
    let (ahead, up) = (Vec3::NEG_Z, Vec3::Y);
    let right = bearing(Vec3::new(30.0, 4.0, 0.0), ahead, up).unwrap();
    assert!((right - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    assert_eq!(slot(bearing(Vec3::NEG_Z * 9.0, ahead, up).unwrap()), 0.5);
    assert!((slot(SPAN * 0.5) - 0.75).abs() < 1e-6);
    let behind = bearing(Vec3::new(-1.0, 0.0, 9.0), ahead, up).unwrap();
    assert_eq!((slot(behind), slot(right)), (0.0, 1.0));
    assert_eq!(bearing(Vec3::Y, ahead, up), None);
    assert_eq!(
        (beside(0.0, Some(0.0)), beside(1.0, Some(1.0))),
        (56.0, 304.0)
    );
    assert_eq!((beside(0.5, Some(0.0)), beside(0.5, None)), (180.0, 180.0));
}
