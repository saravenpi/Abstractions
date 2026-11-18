//! Palette tests: the light and the sky follow the star and the air of the planet, and the home
//! air gives the palette the game had before planets had a record.

use bevy::prelude::*;

use super::air::Air;
use super::celestial::smoothstep;
use super::palette::Palette;
use super::scatter::{luminance, peak, scattered, transmit};
use super::sun::SunDir;
use super::vantage::{LocalSystem, Vantage};
use crate::universe::PlanetRecord;

/// Noon is bright with a lit sun, midnight is dim with a lit moon and stars. A star on the
/// horizon gives orange, weaker light and a warm horizon under a sky still blue above, and
/// a red star tints all of it.
///
/// The moon and the night ambient stay weak enough that a torch still shows at three meters.
#[test]
fn noon_and_midnight_differ() {
    let noon = Palette::at(0.25);
    let night = Palette::at(0.75);
    assert!(noon.sun_lux > 10_000.0 && noon.moon_lux < 1.0);
    assert!(night.sun_lux < 1.0 && (250.0..=450.0).contains(&night.moon_lux));
    assert!(night.ambient_brightness <= 45.0 && (noon.ambient_brightness - 350.0).abs() < 1.0);
    assert!(noon.horizon.length() > night.horizon.length() * 3.0);
    assert!(night.stars > 0.99 && noon.stars < 0.01);
    let (noon, low) = (Palette::lit(0.9, Vec3::ONE), Palette::lit(0.02, Vec3::ONE));
    assert!(low.sun_color.z < 0.3 && noon.sun_color.z > 0.75);
    assert!(low.sun_lux < 0.6 * noon.sun_lux && low.sun_lux > 1000.0);
    assert!(low.horizon.x > low.horizon.z && noon.horizon.z > noon.horizon.x);
    assert!(low.zenith.z > low.zenith.x);
    let red = Palette::lit(0.9, Vec3::new(1.0, 0.5, 0.3));
    assert!(red.sun_color.z < 0.4 && red.zenith.z < noon.zenith.z);
}

/// The palette as it was computed before planets had a record, kept here word for word.
fn frozen(height: f32, star: Vec3) -> Palette {
    let through = star * transmit(height, 1.0);
    let tint = peak(through);
    let day = smoothstep(-0.12, 0.25, height);
    let dusk = smoothstep(-0.2, 0.1, height);
    let blue = scattered(star).lerp(peak(star), 0.1) * peak(star * transmit(height, 0.2));
    let haze = scattered(star).lerp(peak(star), 0.6) * tint.powf(0.7);
    let zenith = Vec3::new(0.015, 0.025, 0.08).lerp(blue * 0.92, day);
    let horizon = Vec3::new(0.05, 0.08, 0.18).lerp(haze * 0.95, dusk);
    let sky = peak(zenith + horizon).lerp(Vec3::ONE, 0.4);
    let full = luminance(star * transmit(1.0, 1.0)).max(1.0e-6);
    Palette {
        zenith,
        horizon,
        ambient: Vec3::new(0.35, 0.45, 0.9).lerp(sky, day),
        ambient_brightness: 40.0 + (350.0 - 40.0) * day,
        sun_color: tint,
        sun_lux: 12_000.0 * (luminance(through) / full).min(1.0) * smoothstep(-0.06, 0.03, height),
        moon_lux: 350.0 * smoothstep(-0.02, 0.3, -height),
        cloud: Vec3::new(0.12, 0.14, 0.24).lerp(tint.lerp(Vec3::ONE, 0.35), day),
        stars: 1.0 - smoothstep(-0.2, 0.0, height),
        sun_dir: Vec3::Y,
        moon_dir: Vec3::NEG_Y,
    }
}

/// Returns a star in the open at the sine `height` above the horizon, straight up in render axes.
fn sun(height: f32) -> SunDir {
    SunDir {
        dir: Vec3::Y,
        height,
        lit: 1.0,
    }
}

/// At the home pressure with a white air tint the palette is the old one bit for bit, at every
/// height of the star and for every star colour, from the ground and as the camera sees it.
#[test]
fn home_air_keeps_the_old_palette() {
    let home = Air::of(&PlanetRecord::TEMPERATE);
    let stars = [
        Vec3::ONE,
        Vec3::new(1.0, 0.5, 0.3),
        Vec3::new(0.7, 0.8, 1.0),
    ];
    for step in 0..=400 {
        let height = step as f32 / 200.0 - 1.0;
        for star in stars {
            let old = frozen(height, star);
            assert_eq!(Palette::through(height, star, &home), old, "{height}");
            assert_eq!(Palette::lit(height, star), old, "{height}");
        }
        let old = frozen(height, Vec3::ONE);
        let seen =
            Palette::through(height, Vec3::ONE, &home).seen(&sun(height), home.share(), Vec3::ONE);
        assert_eq!(seen, old.seen(&sun(height), 1.0, Vec3::ONE), "{height}");
        let under = Palette::under(&sun(height), &Vantage::default(), &LocalSystem::default());
        assert_eq!(under, (old, Vec3::ONE), "{height}");
    }
}
