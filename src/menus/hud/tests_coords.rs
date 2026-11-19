//! HUD tests: the coordinates line turns a planet-fixed direction back into latitude and
//! longitude and writes them with the height.

use bevy::math::DVec3;

use super::coords::{ground, latlon};

/// Returns the direction at `lon` and `lat` radians, as `computer::season::direction` does.
fn direction(lon: f64, lat: f64) -> DVec3 {
    DVec3::new(lat.cos() * lon.cos(), lat.sin(), -lat.cos() * lon.sin())
}

/// Latitude and longitude come back from the direction they make, east and north positive.
#[test]
fn coordinates_invert_the_direction() {
    for (lat, lon) in [
        (12.4_f64, 103.2_f64),
        (-45.0, -170.0),
        (0.0, 0.0),
        (89.0, 179.0),
    ] {
        let dir = direction(lon.to_radians(), lat.to_radians());
        let (back_lat, back_lon) = latlon(dir * 6000.0);
        assert!(
            (back_lat - lat).abs() < 1e-9 && (back_lon - lon).abs() < 1e-9,
            "{lat} {lon}"
        );
    }
}

/// The line reads degrees with one decimal, the hemisphere letters and the height.
#[test]
fn the_coordinates_line_reads_like_a_chart() {
    let mut out = String::new();
    let dir = direction(103.2_f64.to_radians(), 12.4_f64.to_radians());
    ground(&mut out, dir * 6245.0, 6000.0);
    assert_eq!(out, "12.4° N  103.2° E  245 m");
    out.clear();
    ground(
        &mut out,
        DVec3::new(1.0, -0.5, 0.5).normalize() * 5990.0,
        6000.0,
    );
    assert!(out.starts_with("24.1° S  26.6° W  -10 m"), "{out}");
}
