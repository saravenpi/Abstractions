//! Tests of what the sky hides: a bright sky hides the stars and the band even in thin air, a
//! dark one shows them, and twilight shows the bright stars before the faint.

use bevy::prelude::*;

use super::skylight::{BAND, DAY, band, hide_flux, luminance, shown};
use super::starfield::DAY_FLUX;

/// The clear noon sky hides every star under the flux of the star of the system and shows that.
#[test]
fn noon_hides_all_but_the_star_of_the_system() {
    assert_eq!(hide_flux(DAY), DAY_FLUX);
    assert_eq!(hide_flux(5.0 * DAY), DAY_FLUX);
    assert_eq!(shown(0.4, DAY), 0.0);
    assert_eq!(shown(DAY_FLUX + 1.0, DAY), 1.0);
}

/// A dark sky shows the faintest star of the field, and the bright ones come out first at dusk.
#[test]
fn night_shows_all_and_dusk_the_bright_first() {
    assert_eq!(shown(-4.6, 0.002), 1.0);
    let dusk = 0.04;
    assert!(shown(0.0, dusk) > 0.99);
    assert_eq!(shown(-4.0, dusk), 0.0);
    assert!(hide_flux(0.01) > hide_flux(0.001));
}

/// A sky as bright as a clear day hides the band of the galaxy, and a night sky shows it.
#[test]
fn the_band_needs_a_dark_sky() {
    assert_eq!(band(DAY), 0.0);
    assert_eq!(band(BAND.0), 1.0);
    assert!(band(0.0) == 1.0);
    assert!(band(0.012) > band(0.02));
}

/// The luminance of white is one, and of a blue is less than of a green of the same level.
#[test]
fn luminance_follows_the_eye() {
    assert!((luminance(Vec3::ONE) - 1.0).abs() < 1.0e-4);
    assert!(luminance(Vec3::new(0.0, 1.0, 0.0)) > luminance(Vec3::new(0.0, 0.0, 1.0)));
    assert_eq!(luminance(Vec3::ZERO), 0.0);
}
