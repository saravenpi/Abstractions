//! Tests of the star as one real object: a disc of its true angular size up close, gone when it
//! is too small to draw, with a light that falls off with the square of the distance and is
//! nothing between the stars.

use bevy::math::DVec3;

use super::{Body, STAR_POINT, angle};
use crate::computer::SYSTEM_REACH;
use crate::render::observer::Observer;
use crate::render::starfield::sunlight;
use crate::render::vantage::Vantage;
use crate::render::warp::{Drive, Sequence};
use crate::universe::{GLY, Universe};

/// The disc of the star is drawn with its radius over its distance, at any distance, from its
/// true position and size in 32 bits.
#[test]
fn the_disc_of_the_star_is_its_radius_over_its_distance() {
    let universe = Universe::new(7);
    let star = universe.system(universe.home().star).star;
    for distance in [2.0e6, 5.0e5, 4.0e7, 4.0e8, 5.0e9] {
        let at = (DVec3::new(0.6, 0.0, -0.8) * distance).as_vec3();
        let seen = star.radius as f32 / at.length();
        let wanted = (star.radius / distance) as f32;
        assert!((seen / wanted - 1.0).abs() < 1e-4, "{distance}: {seen}");
        assert_eq!(angle(star.radius, distance), wanted);
    }
    assert_eq!(angle(star.radius, 2.0 * star.radius), 0.5);
}

/// From the home planet the star is a disc. Far outside its system it is under a quarter of a
/// pixel and the disc is gone: the star is then the point the star field draws for it.
#[test]
fn the_disc_is_gone_far_outside_the_system() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let star = universe.system(home.id.star).star;
    let seen = angle(star.radius, home.orbit_radius);
    assert!(seen > STAR_POINT, "the star is {seen} rad from home");
    assert!(angle(star.radius, SYSTEM_REACH) < STAR_POINT);
    assert!(angle(star.radius, 1.0e10) < 0.05 * STAR_POINT);
    let angles: Vec<f32> = (1..=60)
        .map(|i| angle(star.radius, home.orbit_radius * f64::from(i)))
        .collect();
    assert!(
        angles.windows(2).all(|pair| pair[1] < pair[0]),
        "{angles:?}"
    );
}

/// The light of the star is whole at the planet, one hundredth ten times as far out, never
/// more closer in, falls without a step, and is nothing at the edge of the system and beyond.
#[test]
fn the_light_of_the_star_falls_with_the_square_of_the_distance() {
    let orbit = 4.0e7;
    assert_eq!(
        (sunlight(orbit, orbit), sunlight(orbit, 0.5 * orbit)),
        (1.0, 1.0)
    );
    assert!((sunlight(orbit, 10.0 * orbit) - 0.01).abs() < 1e-6);
    assert!((sunlight(orbit, 2.0 * orbit) - 0.25).abs() < 1e-6);
    assert_eq!(sunlight(orbit, SYSTEM_REACH), 0.0);
    assert_eq!(sunlight(orbit, 40.0 * SYSTEM_REACH), 0.0);
    let lights: Vec<f32> = (0..=2000)
        .map(|i| {
            sunlight(
                orbit,
                orbit + (SYSTEM_REACH - orbit) * f64::from(i) / 2000.0,
            )
        })
        .collect();
    for pair in lights.windows(2) {
        assert!(pair[1] <= pair[0] && pair[0] - pair[1] < 0.05, "{pair:?}");
    }
}

/// Half way between two stars neither is a disc. The disc of the star left behind, the only
/// one drawn as a body, is far under the size it is drawn from, and so would be the disc of the
/// star ahead, for every star within reach of the home star, the largest ones too. Both are
/// then the points of the star field, and the planets of the system left behind are gone.
#[test]
fn half_way_between_two_stars_neither_is_a_disc() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let from = universe.system(home.id.star).star;
    let stars = universe.stars_near(from.id.galaxy, from.pos, 40.0);
    assert!(stars.len() > 5, "{} stars near home", stars.len());
    for to in stars.iter().filter(|star| star.id != from.id) {
        let half = 0.5 * to.pos.distance(from.pos);
        assert!(
            half > SYSTEM_REACH,
            "two stars {} light-years apart",
            2.0 * half / GLY
        );
        let (behind, ahead) = (angle(from.radius, half), angle(to.radius, half));
        assert!(
            behind < STAR_POINT && ahead < STAR_POINT,
            "{behind} {ahead}"
        );
        let vantage = Vantage {
            observer: Observer {
                eye: (to.pos - from.pos) * 0.5,
                ..Observer::default()
            },
            ..Vantage::default()
        };
        let planet = Body::Planet(home.id).shown(&home, home.position(0.0), &vantage);
        let halo = Body::Halo(home.id).shown(&home, home.position(0.0), &vantage);
        assert!(!planet && !halo, "a planet shows from between the stars");
    }
}

/// No light is drawn on the heading of a warp, at any stage of it: a glow there was a second
/// sun, ahead of the ship all the way, that no star of the universe stood for.
#[test]
fn no_light_stands_ahead_of_a_warp() {
    let mut state = Sequence::default();
    for frame in 0..1200 {
        let drive = Drive {
            charge: if frame < 900 { 1.0 } else { 0.0 },
            speed: (frame as f32 / 600.0).min(1.0),
        };
        state = state.advance(drive, 1.0 / 60.0);
        assert_eq!(
            state.lens(bevy::math::Vec3::NEG_Z).core,
            0.0,
            "frame {frame}"
        );
    }
}
