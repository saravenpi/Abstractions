//! Gas giant tests: the bands and the storm of a giant keep to cloud colours, its material
//! carries them with its rings, its sky seen from the deck is its own, and the world hands it
//! to the sky before anything the sky pulls in could show in front of it.

use std::f32::consts::PI;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Body;
use super::giant::Giant;
use super::material::{BodyMaterial, far};
use crate::render::air::Air;
use crate::render::observer::Observer;
use crate::render::palette::Palette;
use crate::render::smoothstep;
use crate::render::vantage::Vantage;
use crate::universe::{PlanetClass, PlanetRecord, PlanetSpec, RING_OUTER, Rings, Universe, Water};
use crate::world::Globe;
use crate::world::globe::far::{ENTER, FAR, Range, handover, seen};

/// Returns the record of a gas giant.
fn giant() -> PlanetRecord {
    PlanetRecord {
        class: PlanetClass::GasGiant,
        pressure: 5.0,
        ocean: 0.0,
        water: Water::None,
        ..PlanetRecord::TEMPERATE
    }
}

/// Returns the share of the dark tone at latitude `lat` radians, as `bodies.wesl` draws it.
fn band(giant: &Giant, lat: f32) -> f32 {
    let k = giant.bands;
    let wave = (lat * k + giant.phase + 0.8 * (lat * k * 0.43 + 1.3).sin()).sin();
    smoothstep(-0.35, 0.35, wave)
}

/// From pole to pole a giant shows bright zones and dark belts, and every colour it shows,
/// storm included, is a cloud colour: red over green over blue, never green and never the blue
/// of water. Only a gas giant has bands.
#[test]
fn giant_bands_keep_cloud_colours() {
    for seed in (0..64u64).map(|i| i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 5) {
        let giant = Giant::of(&giant(), seed).unwrap();
        assert!((7.0..=13.0).contains(&giant.bands), "{}", giant.bands);
        assert!(giant.spot.is_normalized() && giant.spot.y.abs() <= 0.45);
        assert!((0.98..1.0).contains(&giant.size));
        let shares: Vec<f32> = (0..=200)
            .map(|i| band(&giant, PI * (i as f32 / 200.0 - 0.5)))
            .collect();
        assert!(shares.iter().any(|s| *s > 0.9) && shares.iter().any(|s| *s < 0.1));
        for share in shares {
            let tone = giant.first.lerp(giant.second, share);
            for color in [tone, tone.lerp(giant.storm, 0.5), giant.storm] {
                assert!(
                    color.x >= color.y && color.y >= color.z && color.z > 0.0,
                    "{color}"
                );
            }
        }
    }
    assert_eq!(Giant::of(&PlanetRecord::TEMPERATE, 3), None);
}

/// The material of a ringed giant carries its bands, its storm and the size of its rings in
/// planet radii; the home planet keeps its far colour, no bands and no rings.
#[test]
fn giant_material_draws_bands_and_rings() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let plain = BodyMaterial::planet(&home).body;
    assert_eq!(plain.color, far(&home.record, home.seed).color.extend(0.0));
    assert_eq!((plain.axis.w, plain.rings), (0.0, Vec4::ZERO));
    let mut ringed = home.clone();
    ringed.record = giant();
    ringed.rings = Some(Rings {
        inner: 1.5 * home.radius,
        outer: 2.5 * home.radius,
        density: 0.7,
        color: Vec3::new(0.8, 0.7, 0.6),
    });
    let bands = Giant::of(&ringed.record, ringed.seed).unwrap();
    let body = BodyMaterial::planet(&ringed).body;
    assert_eq!(body.color, bands.first.extend(0.0));
    assert_eq!(body.tone, bands.second.extend(bands.phase));
    assert_eq!(body.axis.w, bands.bands);
    assert!((body.rings - Vec4::new(1.5, 2.5, 0.7, 0.0)).length() < 1.0e-5);
}

/// Inside the air of a giant the sky is its own: its zenith far less blue than home and its
/// horizon warm, starless by day and starry at night, and the clouds close over it.
#[test]
fn giant_sky_from_the_deck() {
    let air = Air::of(&giant());
    let day = Palette::through(0.8, Vec3::ONE, &air);
    let night = Palette::through(-0.6, Vec3::ONE, &air);
    assert!(day.stars < 0.01 && night.stars > 0.99);
    let home = Palette::through(0.8, Vec3::ONE, &Air::HOME);
    let blue = |sky: Vec3| sky.z / sky.x;
    assert!(blue(day.zenith) < 0.5 * blue(home.zenith), "{day:?}");
    assert!(day.horizon.x > day.horizon.z, "{day:?}");
    assert!(air.share() == 1.0 && air.depth() > 2.0);
    assert!(air.cover(0.4) >= 0.8);
}

/// Returns whether the world and the sky leave `planet` to the sky, seen from `away` metres
/// off its centre: the world past its hand-over, the sky when it draws the planet below.
fn handed_over(planet: &PlanetSpec, away: f64) -> (bool, bool) {
    let dir = DVec3::new(0.6, 0.0, 0.8);
    let world = seen(&Globe::of(planet), dir * away, true) == Range::Beyond;
    let pos = planet.position(0.0);
    let vantage = Vantage {
        observer: Observer {
            eye: pos + dir * away,
            ..Observer::default()
        },
        body: usize::from(planet.id.index),
        moon: planet.id.moon,
        ..Vantage::default()
    };
    (world, Body::Planet(planet.id).shown(planet, pos, &vantage))
}

/// For planets of 6, 40, 60 and 150 km, with and without the widest rings, the world and the
/// sky hand the planet over at the same distance, the world still draws its baked sphere for a
/// while, and the planet and its rings, drawn at their true distances up to there, end short of
/// the distance past which the sky pulls bodies in, so no ring or moon shows in front of it.
#[test]
fn the_world_and_the_sky_agree_on_the_handover() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    for radius in [6.0e3, 4.0e4, 6.0e4, 1.5e5] {
        for outer in [None, Some(RING_OUTER * radius)] {
            let rings = outer.map(|outer| Rings {
                inner: 1.5 * radius,
                outer,
                density: 0.5,
                color: Vec3::ONE,
            });
            let planet = PlanetSpec {
                radius,
                rings,
                ..home.clone()
            };
            let at = handover(radius, rings);
            let reach = outer.unwrap_or(radius);
            assert!(at + reach <= f64::from(FAR), "{radius} {outer:?}: {at}");
            assert!(at > ENTER * radius, "{radius} {outer:?}: {at}");
            assert_eq!(handed_over(&planet, at * 0.999), (false, false));
            assert_eq!(handed_over(&planet, at * 1.001), (true, true));
        }
    }
    assert_eq!(handover(home.radius, None), 20.0 * home.radius);
}
