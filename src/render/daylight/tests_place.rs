//! Tests of the light at a place: it is the light the palette gives a camera standing there, a
//! summit is lit before the valley, and day and night ground are lit as themselves whoever looks.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::place::shadow_at;
use super::{PlaceUniform, SKY_EDGE, SUN_EDGE, Sky, daylight};
use crate::render::air::Air;
use crate::render::atmosphere::{air, horizon_dip, shadow};
use crate::render::celestial::smoothstep;
use crate::render::moonlight::{Moonlight, STARLIGHT, moonlit};
use crate::render::moons::Glow;
use crate::render::palette::{NIGHT_AMBIENT_COLOR, Palette};
use crate::render::scatter::luminance;
use crate::render::sky::linear;
use crate::render::sun::SunDir;
use crate::universe::PlanetRecord;

/// Radius and atmosphere height in metres of the planet the tests light.
const GLOBE: (f32, f32) = (6000.0, 4000.0);

/// Returns the air of a planet of `pressure` home atmospheres.
fn air_of(pressure: f64) -> Air {
    Air::of(&PlanetRecord {
        pressure,
        ..PlanetRecord::TEMPERATE
    })
}

/// Returns the direct and the ambient light the palette of the frame gives a camera `altitude`
/// metres up for which the star stands at `height`, built the way the palette system builds it.
fn camera_light(
    air_here: &Air,
    moon: &Moonlight,
    (height, altitude): (f32, f32),
) -> (Vec3, Vec3, f32) {
    let up = DVec3::Y * f64::from(GLOBE.0 + altitude);
    let toward = DVec3::new(
        f64::from((1.0 - height * height).sqrt()),
        f64::from(height),
        0.0,
    );
    let sun = SunDir {
        dir: Vec3::Y,
        height,
        lit: shadow(up, toward, f64::from(GLOBE.0)),
    };
    let left = air(altitude, GLOBE.1) * air_here.share();
    let ground = Palette::through(height + horizon_dip(altitude, GLOBE.0), Vec3::ONE, air_here);
    let palette = moonlit(ground.seen(&sun, left, Vec3::ONE), moon);
    let direct = linear(palette.sun_color).truncate() * palette.sun_lux;
    (direct, palette.ambient, palette.ambient_brightness)
}

/// Returns true when `a` and `b` are the same light to within a hundredth of a percent.
fn close(a: Vec3, b: Vec3) -> bool {
    (a - b).abs().max_element() <= 1.0e-4 * a.abs().max_element().max(1.0)
}

/// Checks that `sky` lights a point `altitude` metres up, at every height of the star, as the
/// palette lights a camera standing there under `moon`: the same direct light, the same ambient
/// brightness on the colour of the rule, and the very same ambient light where it is night.
fn check(sky: &Sky, air_here: &Air, moon: &Moonlight, altitude: f32) {
    for step in 0..=200 {
        let height = step as f32 / 100.0 - 1.0;
        let here = sky.light(height, altitude);
        let (direct, colour, level) = camera_light(air_here, moon, (height, altitude));
        assert!(close(here.sun, direct), "{altitude} {height}");
        let seen = height + horizon_dip(altitude, GLOBE.0);
        let share = smoothstep(SKY_EDGE.0, SKY_EDGE.1, seen);
        let ruled = linear(NIGHT_AMBIENT_COLOR.lerp(sky.day, share)).truncate() * level;
        assert!(close(here.sky, ruled), "{altitude} {height}");
        if share == 0.0 {
            assert!(
                close(here.sky, linear(colour).truncate() * level),
                "{height}"
            );
        }
    }
}

/// At every height of the star, at every altitude, through no air, thin, home and heavy air,
/// with and without a moon, the rule gives a point the light the palette gives a camera standing
/// there, and its day colour is the ambient colour of the palette under a star straight overhead.
#[test]
fn a_place_is_lit_as_a_camera_standing_there() {
    let half = Glow {
        dir: Vec3::X,
        height: 0.5,
        share: 0.4,
    };
    for moon in [Moonlight(None), Moonlight(Some(half))] {
        let starlight = STARLIGHT * (1.0 - moon.0.map_or(0.0, |glow| glow.share));
        for pressure in [0.0, 0.3, 1.0, 3.0] {
            let air_here = air_of(pressure);
            let sky = Sky::of(Vec3::ONE, &air_here, GLOBE, starlight);
            assert_eq!(sky.day, Palette::through(1.0, Vec3::ONE, &air_here).ambient);
            for altitude in [0.0, 120.0, 900.0, 3500.0, 9000.0] {
                check(&sky, &air_here, &moon, altitude);
            }
        }
    }
}

/// The shadow of the planet over a point given by the height of its star and its altitude is the
/// one of the same point given by its position.
#[test]
fn the_shadow_of_a_place_is_the_shadow_of_its_point() {
    for altitude in [0.0, 300.0, 5000.0] {
        for step in 0..=200 {
            let height = step as f32 / 100.0 - 1.0;
            let up = DVec3::Y * f64::from(GLOBE.0 + altitude);
            let toward = DVec3::new(
                f64::from((1.0 - height * height).sqrt()),
                f64::from(height),
                0.0,
            );
            let exact = shadow(up, toward, f64::from(GLOBE.0));
            assert!(
                (shadow_at(height, altitude, GLOBE.0) - exact).abs() < 2.0e-3,
                "{altitude} {height}"
            );
        }
    }
}

/// Past the terminator the valley floor is dark while a summit 300 m above it still catches the
/// star, and no ground is lit once the star is under its own horizon by the edge of the disc:
/// through the air nothing at all, and no more than the share of the open star the air has left
/// free, which the thin air at the height of the place lets by.
#[test]
fn a_summit_is_lit_before_the_valley() {
    let sky = Sky::of(Vec3::ONE, &Air::HOME, GLOBE, STARLIGHT);
    let (valley, summit) = (sky.light(-0.2, 0.0), sky.light(-0.2, 300.0));
    assert_eq!(valley.sun, Vec3::ZERO);
    assert!(luminance(summit.sun) > 500.0, "{}", summit.sun);
    for altitude in [0.0, 50.0, 300.0, 1500.0] {
        let dark = SUN_EDGE.0 - horizon_dip(altitude, GLOBE.0) - 0.01;
        let open = Palette::SPACE_LUX * (1.0 - air(altitude, GLOBE.1));
        let lit = luminance(sky.light(dark, altitude).sun);
        assert!(lit < 1.0 + open, "{altitude}: {lit} against {open}");
        assert_eq!(
            daylight(dark + horizon_dip(altitude, GLOBE.0), Vec3::ONE, 1.0).risen,
            0.0
        );
        let grazed = luminance(sky.light(dark + 0.06, altitude).sun);
        assert!(grazed > 0.0, "{altitude}");
    }
}

/// Ground in the night is dark under a camera in full sunlight and ground in the day is lit
/// under a camera in the night: the light of a place does not depend on the height or the
/// altitude of the camera, which only enter the block as the light the scene already carries.
#[test]
fn the_light_of_a_place_ignores_the_camera() {
    let sky = Sky::of(Vec3::ONE, &Air::HOME, GLOBE, STARLIGHT);
    let (night, day) = (sky.light(-0.5, 0.0), sky.light(0.8, 0.0));
    assert_eq!(night.sun, Vec3::ZERO);
    assert!(luminance(day.sun) > 9000.0 && luminance(day.sky) > 6.0 * luminance(night.sky));
    for eye in [
        (0.6, 2000.0),
        (-0.6, 2000.0),
        (0.1, 10_000.0),
        (-0.9, 30_000.0),
    ] {
        let block = sky.uniform(Vec3::new(0.0, -8000.0, 0.0), Vec3::X, eye);
        let seen = sky.light(eye.0, eye.1);
        assert_eq!(
            (block.eye_sun, block.eye_sky),
            (seen.sun.extend(0.0), seen.sky.extend(0.0))
        );
        assert_eq!(
            (block.centre, block.air.z),
            (Vec4::new(0.0, -8000.0, 0.0, GLOBE.0), 1.0)
        );
        let onto_night = night.sun - block.eye_sun.truncate() + seen.sun;
        let onto_day = day.sun - block.eye_sun.truncate() + seen.sun;
        assert!(
            close(onto_night, Vec3::ZERO) && close(onto_day, day.sun),
            "{eye:?}"
        );
    }
    assert_eq!(PlaceUniform::default().air.z, 0.0);
}
