//! Tests of the star as one real object: a disc of its true angular size up close, gone when it
//! is too small to draw, with a light that falls off with the square of the distance and is
//! nothing between the stars.

use bevy::math::DVec3;

use super::{Body, STAR_POINT, angle};
use crate::render::observer::Observer;
use crate::render::starfield::sunlight;
use crate::render::vantage::Vantage;
use crate::render::warp::{Drive, Sequence};
use crate::universe::{LY, Universe, system_reach};

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
    assert!(angle(star.radius, system_reach(&star)) < STAR_POINT);
    assert!(angle(star.radius, LY) < 0.05 * STAR_POINT);
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
    let reach = 1.0e13;
    let orbit = 4.0e7;
    assert_eq!(
        (
            sunlight(orbit, orbit, reach),
            sunlight(orbit, 0.5 * orbit, reach)
        ),
        (1.0, 1.0)
    );
    assert!((sunlight(orbit, 10.0 * orbit, reach) - 0.01).abs() < 1e-6);
    assert!((sunlight(orbit, 2.0 * orbit, reach) - 0.25).abs() < 1e-6);
    assert_eq!(sunlight(orbit, reach, reach), 0.0);
    assert_eq!(sunlight(orbit, 40.0 * reach, reach), 0.0);
    let lights: Vec<f32> = (0..=2000)
        .map(|i| {
            sunlight(
                orbit,
                orbit + (reach - orbit) * f64::from(i) / 2000.0,
                reach,
            )
        })
        .collect();
    for pair in lights.windows(2) {
        assert!(pair[1] <= pair[0] && pair[0] - pair[1] < 0.05, "{pair:?}");
    }
}

/// Past the reach of its system, whichever way the camera went, the disc of the star left
/// behind is far under the size it is drawn from, so it is the point of the star field, and the
/// planets of the system are gone. The camera is taken that far out along the line to each star
/// within 40 light-years, a thousand reaches short of any of them.
#[test]
fn past_the_reach_of_a_system_the_star_is_not_a_disc() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let from = universe.system(home.id.star).star;
    let reach = system_reach(&from);
    let stars = universe.stars_near(from.id.galaxy, from.pos, 40.0);
    assert!(stars.len() > 5, "{} stars near home", stars.len());
    for to in stars.iter().filter(|star| star.id != from.id) {
        let out = (to.pos - from.pos).normalize() * reach * 1.01;
        assert!(angle(from.radius, out.length()) < STAR_POINT);
        let vantage = Vantage {
            observer: Observer {
                eye: out,
                ..Observer::default()
            },
            ..Vantage::default()
        };
        let planet = Body::Planet(home.id).shown(&home, home.position(0.0), &vantage, reach);
        let halo = Body::Halo(home.id).shown(&home, home.position(0.0), &vantage, reach);
        assert!(!planet && !halo, "a planet shows from outside its system");
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
