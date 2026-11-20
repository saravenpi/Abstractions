//! Tests of the values the water shader reads: the light of the day and the liquid they carry.

use super::*;

/// Noon lights the sun and a night shows the moon only.
#[test]
fn sun_and_moon_visibility_follow_the_day() {
    let fog = FogLook::from_fog(None, Vec3::ONE);
    let noon = WaterUniform::new(&Palette::at(0.25), &Wind::default(), 0.0, &fog);
    let night = WaterUniform::new(&Palette::at(0.75), &Wind::default(), 0.0, &fog);
    assert!(noon.sun.w > 0.99 && noon.moon.w < 0.01);
    assert!(night.sun.w < 0.01 && night.moon.w > 0.9);
}

/// A fresh uniform draws the water of home, and another liquid only changes the liquid values.
#[test]
fn the_uniform_carries_the_liquid() {
    let fog = FogLook::from_fog(None, Vec3::ONE);
    let water = WaterUniform::new(&Palette::at(0.25), &Wind::default(), 0.0, &fog);
    assert_eq!(water.shallow, Vec4::new(0.03, 0.55, 0.48, 0.0));
    assert_eq!(water.deep, Vec4::new(0.002, 0.04, 0.16, 1.0));
    assert_eq!(water.foam, Vec4::new(0.95, 0.98, 1.0, 0.0));
    let lava = Liquid {
        glow: 4.0,
        cover: 1.0,
        ..Liquid::WATER
    };
    let hot = water.of(&lava);
    assert_eq!((hot.shallow.w, hot.foam.w), (4.0, 1.0));
    assert_eq!(
        (hot.sun, hot.wave_a, hot.zenith),
        (water.sun, water.wave_a, water.zenith)
    );
    assert_eq!((hot.far, hot.blend), (water.far, blend::packed()));
    let home = Lands::of(&PlanetRecord::TEMPERATE);
    assert_eq!(water.far, home.sea_shown().extend(1.0));
}
