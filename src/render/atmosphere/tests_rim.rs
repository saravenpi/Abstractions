//! Rim tests: the halo of a body is as thick as its air from near, never thinner than a pixel and
//! a half from afar, never fatter than it ever was, and its limb is where the planet's edge shows.

use super::rim::{HALO_SCALE, RIM, halo_scale, limb};
use crate::universe::{PlanetSpec, Universe};

/// Returns the home planet of seed 7.
fn home() -> PlanetSpec {
    let universe = Universe::new(7);
    universe.planet(universe.home())
}

/// From the hand-over the halo of the home planet is the top of its air, a few hundredths of its
/// radius and less than the old fixed shell of 1.08 radii.
#[test]
fn near_the_halo_is_as_thick_as_the_air() {
    let planet = home();
    let air = (planet.atmosphere_height / planet.radius) as f32;
    assert!(air > 0.0 && air < HALO_SCALE - 1.0, "{air}");
    let scale = halo_scale(&planet, 1.0 / 1.5);
    assert!(
        (scale - (1.0 + air.max(RIM * 1.5))).abs() < 1.0e-6,
        "{scale}"
    );
}

/// From afar the halo stays at least [`RIM`] radians wide, and it never grows past the old
/// shell, however small the body is on the screen.
#[test]
fn far_the_halo_keeps_a_steady_width() {
    let planet = home();
    for angle in [0.1, 0.02, 0.01] {
        let wide = (halo_scale(&planet, angle) - 1.0) * angle;
        assert!(wide >= RIM - 1.0e-6 || halo_scale(&planet, angle) >= HALO_SCALE - 1.0e-6);
    }
    assert!((halo_scale(&planet, 1.0e-6) - HALO_SCALE).abs() < 1.0e-6);
}

/// The limb is where a line of sight that grazes the planet meets the shell: its facing is the
/// cosine of the angle whose sine is one over the radius of the shell.
#[test]
fn the_limb_is_where_the_planet_grazes() {
    for scale in [1.01_f32, 1.04, HALO_SCALE] {
        let grazing = (1.0 / scale).asin().cos();
        assert!((limb(scale) - grazing).abs() < 1.0e-5, "{scale}");
    }
    assert_eq!(limb(1.0), 0.0);
}
