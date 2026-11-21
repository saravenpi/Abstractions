//! Tests of the hot skin and of the edge of the glow: a nose that radiates its heat away glows in
//! the colour of its temperature, and under the cool speed of an air nothing glows.

use super::tune::HEATING;
use super::{Gas, cool_speed, entry, flux, incandescence, skin, temperature};

/// Density of the air of the home planet at sea level in kilograms per cubic metre.
const HOME: f64 = 1.2;

/// Density at sea level of a planet with a tenth of the home air.
const THIN: f64 = 0.12;

/// Density at sea level of a planet with three times the home air.
const THICK: f64 = 3.6;

/// The skin of a nose that starts to glow sits near 1,200 K and shows a dull red; at the full
/// glow it sits near 2,550 K and shows orange; the colour read back from a heat share matches.
#[test]
fn the_skin_glows_in_the_colour_of_its_temperature() {
    let (low, full) = HEATING;
    assert!(
        (temperature(low) - 1200.0).abs() < 30.0,
        "{}",
        temperature(low)
    );
    assert!(
        (temperature(full) - 2540.0).abs() < 40.0,
        "{}",
        temperature(full)
    );
    let dull = entry(HOME, 730.0, Gas::NitrogenOxygen).skin;
    let bright = entry(HOME, 2000.0, Gas::NitrogenOxygen).skin;
    assert_eq!(dull.x, 1.0);
    assert!(dull.y < 0.3 && dull.z < 0.05, "{dull}");
    assert!(
        bright.y > 0.5 && bright.z > 0.1 && bright.z < bright.y,
        "{bright}"
    );
    assert!(skin(0.0).distance(incandescence(temperature(low) as f32)) < 1.0e-3);
    assert!(skin(1.0).y > 0.5);
}

/// Up to the speed that keeps the nose dark the air makes no glow, and that speed has no bound
/// without air.
#[test]
fn the_cool_speed_is_the_edge_of_the_glow() {
    for density in [THIN, HOME, THICK] {
        let edge = cool_speed(density);
        assert!((flux(density, edge) / HEATING.0 - 1.0).abs() < 1.0e-9);
        assert_eq!(entry(density, edge * 0.99, Gas::NitrogenOxygen).heat, 0.0);
        assert!(entry(density, edge * 1.5, Gas::NitrogenOxygen).heat > 0.0);
    }
    assert!(cool_speed(HOME) > 650.0 && cool_speed(HOME) < 800.0);
    assert_eq!(cool_speed(0.0), f64::INFINITY);
}
