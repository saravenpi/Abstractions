//! Weather tests: calm weather leaves the sky of every class as it was, rain darkens and lowers
//! the clouds, dust stains the sky and closes the fog in, and the reported cover drives the shell.

use bevy::prelude::*;

use super::air::Air;
use super::dome::cloud_cover;
use super::palette::Palette;
use super::scatter::luminance;
use super::weather::{LocalWeather, WeatherLook};
use crate::universe::{PlanetClass, PlanetRecord};

/// Returns the air of the home record with another class.
fn air(class: PlanetClass) -> Air {
    Air::of(&PlanetRecord {
        class,
        ..PlanetRecord::TEMPERATE
    })
}

/// Without a weather report every class keeps the sky of 0.8.1 bit for bit: the same palette
/// at every height of the star and for every star colour, the same cloud cover, cloud base and fog.
#[test]
fn calm_weather_keeps_the_old_sky() {
    let stars = [
        Vec3::ONE,
        Vec3::new(1.0, 0.5, 0.3),
        Vec3::new(0.7, 0.8, 1.0),
    ];
    for class in PlanetClass::ALL {
        let air = air(class);
        let look = LocalWeather::default().look(&air);
        assert_eq!(look, WeatherLook::of(&air), "{class:?}");
        assert_eq!(
            (look.base(1200.0), look.sight()),
            (1200.0, 1.0),
            "{class:?}"
        );
        for step in 0..=200 {
            let height = step as f32 / 100.0 - 1.0;
            for star in stars {
                let old = Palette::through(height, star, &air);
                assert_eq!(look.sky(old), old, "{class:?} {height}");
            }
            let drift = cloud_cover(step as f32 / 200.0);
            assert_eq!(look.cover(&air, drift), air.cover(drift), "{class:?}");
        }
    }
    let home = LocalWeather::default().look(&Air::HOME);
    assert_eq!(
        home.sky(Palette::lit(0.4, Vec3::ONE)),
        Palette::lit(0.4, Vec3::ONE)
    );
}

/// A storm makes darker clouds, a lower cloud base, a greyer sky and a closer fog; a reported
/// cover above the class mean thickens the shell.
#[test]
fn rain_darkens_and_lowers_the_clouds() {
    let calm = WeatherLook::of(&Air::HOME);
    let storm = WeatherLook {
        haze: 1.0,
        cover: 0.9,
        ..calm
    };
    let noon = Palette::lit(0.8, Vec3::ONE);
    let wet = storm.sky(noon);
    assert!(luminance(wet.cloud) < 0.5 * luminance(noon.cloud));
    assert!(storm.base(1000.0) < 700.0 && storm.base(1000.0) > 600.0);
    assert!(storm.sight() < 0.6);
    let spread = |c: Vec3| c.max_element() - c.min_element();
    assert!(spread(wet.zenith) < 0.6 * spread(noon.zenith));
    assert!((luminance(wet.zenith) - luminance(noon.zenith)).abs() < 1.0e-4);
    assert!(storm.cover(&Air::HOME, 0.5) > calm.cover(&Air::HOME, 0.5));
}

/// Dust turns the blue sky ochre at the same brightness, stains the clouds and shortens the fog
/// to well under a third of its clear reach.
#[test]
fn dust_stains_the_sky_and_closes_the_fog() {
    let dusty = WeatherLook {
        dust: 1.0,
        ..WeatherLook::of(&air(PlanetClass::Desert))
    };
    let noon = Palette::lit(0.8, Vec3::ONE);
    let sky = dusty.sky(noon);
    assert!(noon.zenith.z > noon.zenith.x && sky.zenith.x > sky.zenith.z);
    assert!(sky.horizon.x > sky.horizon.z && sky.cloud.x > sky.cloud.z);
    assert!((luminance(sky.zenith) - luminance(noon.zenith)).abs() < 1.0e-4);
    assert!(dusty.sight() < 0.3);
    assert_eq!(sky.sun_lux, noon.sun_lux);
}
