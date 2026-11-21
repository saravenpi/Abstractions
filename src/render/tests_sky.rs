//! Sky tests: the day sky, the light and the stars follow the pressure and the tint of the air
//! of the planet the camera stands on.

use bevy::prelude::*;

use super::air::Air;
use super::palette::Palette;
use super::scatter::luminance;
use super::sky::linear;
use super::sun::SunDir;
use crate::universe::{PlanetClass, PlanetRecord};

/// Returns a star in the open at the sine `height` above the horizon, straight up in render axes.
fn sun(height: f32) -> SunDir {
    SunDir {
        dir: Vec3::Y,
        height,
        lit: 1.0,
    }
}

/// Returns the palette a camera at sea level gets on a planet of this record under a white
/// star whose height has the sine `height`, the way the palette system builds it.
fn standing(record: &PlanetRecord, height: f32) -> Palette {
    let air = Air::of(record);
    Palette::through(height, Vec3::ONE, &air).seen(&sun(height), air.share(), Vec3::ONE)
}

/// Returns the home record with another class and pressure.
fn world(class: PlanetClass, pressure: f64) -> PlanetRecord {
    PlanetRecord {
        class,
        pressure,
        ..PlanetRecord::TEMPERATE
    }
}

/// As the pressure falls from the home value to nothing the noon sky only gets darker, the
/// ambient light falls to what is left in space and the stars come out.
#[test]
fn day_sky_darkens_as_pressure_falls() {
    let pressures = [1.0, 0.8, 0.6, 0.4, 0.25, 0.12, 0.05, 0.02];
    let skies: Vec<Palette> = pressures
        .iter()
        .map(|&pressure| standing(&world(PlanetClass::Island, pressure), 0.9))
        .collect();
    for pair in skies.windows(2) {
        assert!(luminance(pair[1].zenith) < luminance(pair[0].zenith));
        assert!(luminance(pair[1].horizon) < luminance(pair[0].horizon));
        assert!(pair[1].ambient_brightness < pair[0].ambient_brightness);
        assert!(pair[1].stars > pair[0].stars);
        assert!(pair[1].sun_lux >= pair[0].sun_lux);
    }
    let thin = skies[pressures.len() - 1];
    assert!(luminance(thin.zenith) < 0.03 * luminance(skies[0].zenith));
    assert!(thin.ambient_brightness < Palette::DAY_AMBIENT * (Palette::SPACE_AMBIENT + 0.05));
}

/// Without air the sky is black at noon, the stars are out, the ambient light is the one of
/// space, and the star is as white and as strong on the horizon as overhead.
#[test]
fn no_air_gives_a_black_sky_and_stars_at_noon() {
    let full = Palette::SPACE_LUX;
    for pressure in [0.0, 0.004, 0.01] {
        let record = world(PlanetClass::Barren, pressure);
        let (noon, low) = (standing(&record, 0.95), standing(&record, 0.05));
        assert_eq!((noon.zenith, noon.horizon), (Vec3::ZERO, Vec3::ZERO));
        assert_eq!((noon.stars, low.stars), (1.0, 1.0));
        assert_eq!(
            noon.ambient_brightness,
            Palette::DAY_AMBIENT * Palette::SPACE_AMBIENT
        );
        assert_eq!((noon.sun_color, low.sun_color), (Vec3::ONE, Vec3::ONE));
        assert_eq!((noon.sun_lux, low.sun_lux), (full, full));
    }
    let home = standing(&PlanetRecord::TEMPERATE, 0.05);
    assert!(home.sun_color.z < 0.5 && home.sun_lux < 0.7 * full);
}

/// Thicker air dims and reddens a low star more than the home air does and whitens the
/// horizon, a high star keeps most of its light, and no pressure, however high, takes a colour
/// out of range or the star away.
#[test]
fn thick_air_reddens_and_dims_a_low_star() {
    let home = standing(&PlanetRecord::TEMPERATE, 0.1);
    let white = |sky: Vec3| sky.min_element() / sky.max_element();
    let clear = white(standing(&PlanetRecord::TEMPERATE, 0.95).horizon);
    let mut last = home;
    for pressure in [1.5, 2.0, 3.0, 5.0] {
        let record = world(PlanetClass::Island, pressure);
        let (low, noon) = (standing(&record, 0.1), standing(&record, 0.95));
        assert!(low.sun_lux < last.sun_lux && low.sun_color.z < last.sun_color.z);
        assert!(low.sun_color.y < last.sun_color.y && low.sun_color.x == 1.0);
        assert!(noon.sun_lux > 0.8 * Palette::SPACE_LUX && noon.sun_lux < Palette::SPACE_LUX);
        assert!(white(noon.horizon) > clear, "{pressure}");
        for sky in [low.zenith, low.horizon, noon.zenith, noon.horizon] {
            assert!(
                sky.min_element() >= 0.0 && sky.max_element() <= 1.0,
                "{sky}"
            );
        }
        last = low;
    }
    assert!(last.sun_lux > 0.2 * home.sun_lux);
    let crushing = standing(&world(PlanetClass::Island, 50.0), 0.1);
    assert!(crushing.sun_lux > 0.9 * last.sun_lux);
}

/// A white air tint leaves the sky blue. A desert sky is butterscotch, a lava sky smoky red and
/// an ice sky a paler blue, and the ambient light, the fog colour and the clouds follow. The
/// light of the star itself is not tinted.
#[test]
fn air_tint_stains_the_sky() {
    let sky = |class: PlanetClass| standing(&world(class, 1.0), 0.9);
    let pale = |sky: Vec3| sky.min_element() / sky.max_element();
    let (home, ocean) = (sky(PlanetClass::Island), sky(PlanetClass::Ocean));
    assert!(home.zenith.z > 2.0 * home.zenith.x && home.ambient.z > home.ambient.x);
    assert_eq!(home, ocean);
    let desert = sky(PlanetClass::Desert);
    assert!(desert.zenith.x > desert.zenith.y && desert.zenith.y > desert.zenith.z);
    assert!(desert.horizon.x > desert.horizon.y && desert.horizon.y > desert.horizon.z);
    assert!(desert.ambient.x > desert.ambient.z && desert.cloud.x > desert.cloud.z);
    assert!(luminance(desert.horizon) > luminance(desert.zenith));
    let lava = sky(PlanetClass::Lava);
    assert!(lava.zenith.x > 1.6 * lava.zenith.y && lava.zenith.y > lava.zenith.z);
    assert!(luminance(lava.zenith) < 0.5 && pale(lava.zenith) < pale(desert.zenith));
    let ice = sky(PlanetClass::Ice);
    assert!(ice.zenith.z > ice.zenith.x && pale(ice.zenith) > 1.3 * pale(home.zenith));
    for tinted in [desert, lava, ice] {
        assert_eq!(tinted.sun_color, home.sun_color);
        assert_eq!(tinted.sun_lux, home.sun_lux);
    }
}

/// At noon the dusty zenith of a desert is as bright in linear light as the blue zenith it
/// replaces, and no more than a quarter brighter; its horizon stays short of white; and it is
/// the colour of its air, more red than green and more green than blue by a clear step.
#[test]
fn desert_sky_is_butterscotch_not_mud() {
    let light = |sky: Vec3| luminance(linear(sky).truncate());
    let home = standing(&PlanetRecord::TEMPERATE, 1.0);
    let desert = standing(&world(PlanetClass::Desert, 1.0), 1.0);
    let (dusty, clean) = (light(desert.zenith), light(home.zenith));
    assert!(
        dusty >= clean && dusty < 1.25 * clean,
        "{dusty} against {clean}"
    );
    assert!((0.85..0.95).contains(&desert.horizon.max_element()));
    let zenith = desert.zenith;
    assert!(zenith.x > 0.54 && zenith.y < 0.87 * zenith.x && zenith.z < 0.83 * zenith.y);
    let barren = standing(&world(PlanetClass::Barren, 1.0), 1.0);
    assert_eq!((barren.zenith, barren.horizon), (home.zenith, home.horizon));
}
