//! Season tests: autumn by latitude and season, the tint of leaves against rock and sand, snow
//! only where it freezes, and the home planet looking as it did when the game starts.

use std::f32::consts::{FRAC_PI_4, TAU};

use bevy::math::DVec3;
use bevy::prelude::*;

use super::season::{SeasonLook, autumn, repaint, snow, tint};
use super::vantage::Nearby;
use crate::climate::season as phase;
use crate::climate::{Setting, Spot};
use crate::universe::{PlanetSpec, Universe};

/// Ground colours of the world tables in sRGB: grass of every biome, then rock, sand and water.
const GRASS: [Vec3; 5] = [
    Vec3::new(0.16, 0.46, 0.18),
    Vec3::new(0.52, 0.68, 0.26),
    Vec3::new(0.62, 0.70, 0.42),
    Vec3::new(0.14, 0.70, 0.22),
    Vec3::new(0.25, 0.28, 0.20),
];

/// Ground colours no season touches, in sRGB.
const BARE: [Vec3; 4] = [
    Vec3::new(0.48, 0.46, 0.45),
    Vec3::new(0.82, 0.74, 0.52),
    Vec3::new(0.12, 0.22, 0.32),
    Vec3::new(0.50, 0.34, 0.20),
];

/// Returns the home planet of seed 7.
pub(super) fn home() -> PlanetSpec {
    let universe = Universe::new(7);
    let body = universe.home();
    Nearby::read(&universe, body.star).system.planets[usize::from(body.index)].clone()
}

/// Returns a spot on flat ground at `latitude` radians and `altitude` metres.
pub(super) fn spot(latitude: f32, altitude: f32) -> Spot {
    Spot {
        dir: DVec3::new(f64::from(latitude.cos()), f64::from(latitude.sin()), 0.0),
        altitude,
        inland: 0.0,
        slope: Vec2::ZERO,
    }
}

/// Leaves turn in the autumn of each hemisphere, half a year apart, the more so the farther
/// from the equator and the more the planet leans, and never without a tilt.
#[test]
fn autumn_follows_latitude_and_season() {
    let tilt = 0.4;
    assert!(autumn(FRAC_PI_4, 0.4, tilt) > 0.95);
    assert!(autumn(-FRAC_PI_4, 0.9, tilt) > 0.95);
    for season in [0.0, 0.1, 0.65, 0.8] {
        assert_eq!(autumn(FRAC_PI_4, season, tilt), 0.0, "{season}");
    }
    assert_eq!(autumn(-FRAC_PI_4, 0.4, tilt), 0.0);
    for step in 0..100 {
        let season = step as f32 / 100.0;
        assert_eq!(autumn(0.0, season, tilt), 0.0);
        assert_eq!(autumn(1.0, season, 0.0), 0.0);
        assert!(autumn(0.2, season, tilt) <= autumn(0.6, season, tilt) + 1.0e-6);
    }
}

/// Autumn yellows and reddens grass and leaves, snow whitens them, and rock, sand, water and
/// wood keep their colour under any season.
#[test]
fn tint_turns_leaves_not_the_rest() {
    let fall = SeasonLook {
        autumn: 1.0,
        snow: 0.0,
    };
    let winter = SeasonLook {
        autumn: 0.0,
        snow: 1.0,
    };
    for grass in GRASS {
        let turned = tint(grass, fall);
        assert!(
            turned.x - turned.y > grass.x - grass.y + 0.05,
            "{grass} {turned}"
        );
        assert!(
            tint(grass, winter).min_element() > grass.min_element() + 0.1,
            "{grass}"
        );
        assert_eq!(tint(grass, SeasonLook::NONE), grass);
    }
    for bare in BARE {
        assert_eq!(tint(bare, fall), bare);
        assert_eq!(tint(bare, winter), bare);
    }
}

/// Snow lies only below freezing, grows as a seasonal temperature swings toward winter and
/// never lies without air.
#[test]
fn snow_only_where_it_freezes_and_grows_toward_winter() {
    assert_eq!((snow(300.0), snow(275.0)), (0.0, 0.0));
    assert_eq!(snow(268.0), 1.0);
    let temperature = |season: f32| 276.0 + 8.0 * (TAU * season).cos();
    let cover: Vec<f32> = (0..=10)
        .map(|i| snow(temperature(i as f32 / 20.0)))
        .collect();
    assert_eq!(cover[0], 0.0);
    assert!(cover.windows(2).all(|pair| pair[1] >= pair[0]) && cover[10] == 1.0);
    let mut setting = Setting::of(&home());
    setting.temperature = 250.0;
    assert_eq!(SeasonLook::at(&setting, &spot(0.5, 0.0), 0.0).snow, 1.0);
    setting.pressure = 0.0;
    assert_eq!(SeasonLook::at(&setting, &spot(0.5, 0.0), 0.0).snow, 0.0);
}

/// Kodya leans like the Earth and its poles freeze, but at the spring equinox the ground of
/// its warm and temperate belts, up to 35 degrees north where the base stands, keeps every
/// colour within one byte of 0.8.1 on the low ground around the base.
#[test]
fn home_ground_looks_as_today_at_the_equinox() {
    let home = home();
    let setting = Setting::of(&home);
    let cases = (0..=35)
        .step_by(5)
        .flat_map(|d| [0.0, 120.0, 300.0].map(|a| (0.75_f32, d, a)));
    for (season, degrees, altitude) in cases {
        let here = spot((degrees as f32).to_radians(), altitude);
        let look = SeasonLook::at(&setting, &here, season).steps();
        for color in GRASS.iter().chain(&BARE) {
            let shift = (tint(*color, look) - *color).abs().max_element();
            assert!(shift < 1.0 / 255.0, "{degrees} {season} {altitude}");
        }
    }
}

/// Autumn and the start of winter turn the greens of Kodya's middle latitudes, in the north
/// half a year after the south, and leave its rock and sand alone.
#[test]
fn autumn_turns_home_greens() {
    let home = home();
    let setting = Setting::of(&home);
    for (latitude, season) in [(45.0f32, 0.4), (45.0, 0.5), (-45.0, 0.9)] {
        let look = SeasonLook::at(&setting, &spot(latitude.to_radians(), 0.0), season);
        assert!(look.autumn > 0.4, "{latitude} {season}");
        for grass in GRASS {
            let turned = tint(grass, look);
            assert!(
                turned.x - turned.y > grass.x - grass.y + 0.02,
                "{grass} {season}"
            );
        }
        for bare in BARE {
            assert_eq!(tint(bare, look), bare);
        }
    }
}

/// The season goes once round per orbit, and the texture of a season without effect keeps its bytes.
#[test]
fn phase_goes_round_and_none_repaints_nothing() {
    let home = home();
    let quarter = phase(&home, home.orbit_seconds / 4.0) - phase(&home, 0.0);
    assert!((quarter.rem_euclid(1.0) - 0.25).abs() < 1.0e-3);
    let original: Vec<u8> = (0..=255u8)
        .flat_map(|v| [v, v.wrapping_mul(7), 255 - v, 255])
        .collect();
    let mut out = vec![0; original.len()];
    repaint(&original, &mut out, SeasonLook::NONE);
    assert_eq!(out, original);
    repaint(
        &original,
        &mut out,
        SeasonLook {
            autumn: 1.0,
            snow: 0.5,
        },
    );
    assert_ne!(out, original);
}
