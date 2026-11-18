//! Moon tests: a moon in the body list goes where its planet goes, shows the phase the star
//! gives it, and lights the night where it stands; a planet without moons keeps the old moon.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::moonlight::{Moonlight, moonlit};
use super::moons::{moonlight, phase};
use super::observer::Observer;
use super::palette::Palette;
use super::vantage::{Nearby, Vantage};
use crate::universe::{BodyId, PlanetSpec, Universe};

/// Returns the home system of seed 7 with one moon added around the home planet, 30 planet
/// radii out, and the rank of that planet.
fn with_moon() -> (Nearby, u8) {
    let universe = Universe::new(7);
    let home = universe.home();
    let mut near = Nearby::read(&universe, home.star);
    let planet = near.planet(home.index).unwrap().clone();
    let moon = PlanetSpec {
        id: BodyId {
            moon: 1,
            ..planet.id
        },
        radius: planet.radius * 0.3,
        orbit_radius: planet.radius * 30.0,
        orbit_seconds: 5000.0,
        orbit_phase: 1.0,
        host: Some(planet.orbit()),
        moons: 0,
        ..planet
    };
    near.moons.push(moon);
    (near, home.index)
}

/// Returns a vantage standing on the planet at `pos`, looking out along `up`, render axes
/// being system axes.
fn standing(near: &Nearby, index: u8, up: DVec3, time: f64) -> Vantage {
    let planet = near.planet(index).unwrap();
    let center = planet.position(time);
    let observer = Observer {
        eye: center + up * planet.radius,
        center,
        spin: DQuat::IDENTITY,
        to_render: DQuat::IDENTITY,
    };
    Vantage::of(&observer, near, planet, Vec3::ZERO)
}

/// The moon of the body list is where its planet is plus its own orbit at every time, it is
/// found by its rank, and it belongs to the family of its planet and of no other.
#[test]
fn moons_follow_their_planet() {
    let (near, index) = with_moon();
    let moon = near.body(index, 1).unwrap();
    let planet = near.planet(index).unwrap();
    assert_eq!(moon.id.moon, 1);
    for time in [0.0, 1234.5, 98_765.0, 3.0e7] {
        let apart = moon.position(time) - planet.position(time);
        assert!((apart.length() - moon.orbit_radius).abs() < 1.0e-3 * moon.orbit_radius);
    }
    assert!(near.family(index).any(|body| body.id == moon.id));
    assert_eq!(near.family(index).count(), 2);
    let other = (index + 1) % near.system.planets.len() as u8;
    assert!(near.family(other).all(|body| !body.id.is_moon()));
    assert_eq!(near.body(index, 9).unwrap().id, planet.id);
}

/// A body is full seen from between it and the star, new seen from behind it, and half lit
/// seen from the side, whatever its distance.
#[test]
fn phase_follows_the_star() {
    for distance in [1.0e7, 1.5e11] {
        let body = DVec3::new(distance, 0.0, 0.0);
        assert!((phase(body, body * 0.5) - 1.0).abs() < 1.0e-6);
        assert!(phase(body, body * 2.0).abs() < 1.0e-6);
        let side = body + DVec3::Z * 1.0e5;
        assert!((phase(body, side) - 0.5).abs() < 1.0e-3);
        assert!(phase(body, body * 0.5 + DVec3::Z * distance * 0.2) > 0.9);
    }
}

/// Under a moon high in the sky the night light comes from where it stands, never more than its
/// phase allows; over one orbit it waxes and wanes, and below the horizon it gives no light.
/// A planet without moons keeps the moon opposite the star, its palette untouched.
#[test]
fn moonlight_follows_real_moons() {
    let (near, index) = with_moon();
    let moon = near.body(index, 1).unwrap().clone();
    let at = |time: f64| (moon.position(time) - moon.host.unwrap().at(time)).normalize();
    let mut shares = Vec::new();
    for step in 0..40 {
        let time = f64::from(step) * moon.orbit_seconds / 40.0;
        let vantage = standing(&near, index, at(time), time);
        let glow = moonlight(&vantage, &near, time).unwrap();
        let toward = (moon.position(time) - vantage.observer.eye).normalize();
        assert!(glow.height > 0.99 && glow.dir.as_dvec3().dot(toward) > 0.9999);
        assert!(glow.share <= 3.0 * phase(moon.position(time), vantage.observer.eye) + 1.0e-6);
        shares.push(glow.share);
    }
    let most = shares.iter().copied().fold(0.0, f32::max);
    assert!(
        most > 0.0 && shares.iter().any(|share| *share < 0.2 * most),
        "{shares:?}"
    );
    let below = standing(&near, index, -at(0.0), 0.0);
    assert_eq!(moonlight(&below, &near, 0.0).unwrap().share, 0.0);
    let glow = moonlight(&standing(&near, index, at(0.0), 0.0), &near, 0.0).unwrap();
    let palette = Palette::at(0.75);
    let lit = moonlit(palette, &Moonlight(Some(glow)));
    assert_eq!(lit.moon_dir, glow.dir);
    assert!(lit.moon_lux > 0.0 && lit.moon_lux <= palette.moon_lux * 3.0);
    let alone = Nearby::read(&Universe::new(7), near.id);
    let home = standing(&alone, index, at(0.0), 0.0);
    assert_eq!(moonlight(&home, &alone, 0.0), None);
    assert_eq!(moonlit(palette, &Moonlight(None)), palette);
}
