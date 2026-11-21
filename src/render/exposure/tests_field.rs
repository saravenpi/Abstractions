//! Tests of the brightness of the lit scene: the reference day shows mean ground near 0.8, a night
//! twenty times darker, and that hides the stars by day under a whole sky, not by night, and not
//! under no air, as in space past a sunlit hull.

use bevy::camera::Exposure;

use super::field::shown;
use super::target_ev;
use super::tune::REFERENCE_LUX;
use crate::render::palette::Palette;
use crate::render::skylight::{DAY, FIELD_DAY, band, hide_flux, veil};
use crate::render::starfield::DAY_FLUX;

/// Lux of a night lit by the stars alone: the night ambient and the starlight, 100 cd/m2, times pi.
const NIGHT_LUX: f32 = 314.0;

/// At the reference day the mean ground shows at the luminance the veil is tuned for, within a
/// twentieth, and no lux shows nothing.
#[test]
fn the_reference_day_shows_mean_ground_near_the_tuned_level() {
    let day = shown(REFERENCE_LUX, Exposure::EV100_BLENDER);
    assert!((day / FIELD_DAY - 1.0).abs() < 0.05, "{day}");
    assert_eq!(shown(0.0, Exposure::EV100_BLENDER), 0.0);
    assert_eq!(shown(-5.0, Exposure::EV100_BLENDER), 0.0);
}

/// Under a whole sky a sunlit ground hides every star but the star of the system and the band
/// of the galaxy, as a day sky does; the ground of a starlit night hides no star the night sky
/// shows, and leaves the band whole.
#[test]
fn a_sunlit_ground_hides_the_stars_and_a_night_does_not() {
    let day = veil(shown(REFERENCE_LUX, Exposure::EV100_BLENDER), 1.0);
    assert!(hide_flux(day) > DAY_FLUX - 0.2, "{day}");
    assert_eq!(band(day), 0.0);
    let night = veil(shown(NIGHT_LUX, target_ev(NIGHT_LUX)), 1.0);
    assert!(night < 0.002, "{night}");
    assert_eq!(band(night), 1.0);
    assert_eq!(veil(0.0, 1.0), 0.0);
    assert!(veil(2.0 * FIELD_DAY, 1.0) > DAY);
}

/// In space a sunlit hull meters as a full day, the bug of release 0.11.0, yet under no air it
/// hides no star and leaves the band whole, however bright it shows; thinner air hides less.
#[test]
fn a_sunlit_hull_under_no_air_hides_nothing() {
    let lux = Palette::SPACE_LUX;
    let hull = shown(lux, target_ev(lux));
    assert!(hull > 0.6 * FIELD_DAY, "{hull}");
    for bright in [hull, 2.0 * FIELD_DAY, 10.0 * FIELD_DAY] {
        assert_eq!(veil(bright, 0.0), 0.0, "{bright}");
        assert_eq!(band(veil(bright, 0.0)), 1.0, "{bright}");
    }
    assert!(veil(hull, 0.1) < veil(hull, 0.5));
    assert!(veil(hull, 0.5) < veil(hull, 1.0));
}
