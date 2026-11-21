//! Tests of the light a base stands in: the star climbs high over it on a planet that turns and on one locked
//! to its star, where it stands on the day side, and a new game opens with the star in the sky.

use std::sync::Arc;

use bevy::math::{DQuat, DVec3};

use super::Base;
use crate::render::{Observer, START_PHASE, catch_up};
use crate::universe::{Daystar, LIT, PlanetSpec, Universe, sun};
use crate::world::{Globe, Link};

/// Sine the star may stay short of [`LIT`] over a base: the base stands a few kilometres from the shore it
/// was looked for around.
const SLACK: f64 = 0.01;

/// Returns the home planet of the world of `seed`.
fn home(seed: u64) -> PlanetSpec {
    let universe = Universe::new(seed);
    universe.planet(universe.home())
}

/// Returns the home planet of seed 7 as it would be turning once in 30 hours.
fn turning() -> PlanetSpec {
    PlanetSpec {
        day_seconds: 108_000.0,
        ..home(7)
    }
}

/// Returns the base of `planet` and the planet position a new game lands at on it.
fn founded(planet: &PlanetSpec) -> (Base, DVec3) {
    let globe = Arc::new(Globe::of(planet));
    (Base::on(&globe, planet.id), Base::landing(&globe))
}

/// Returns the observer of a camera at the planet position `at` of `planet` at `time` seconds, as the sky
/// reads it on a surface anchor.
fn observer(planet: &PlanetSpec, at: DVec3, time: f64) -> Observer {
    let link = Link::orbiting(planet, time);
    Observer {
        eye: link.to_anchor(at),
        center: link.offset,
        spin: link.turn,
        to_render: DQuat::IDENTITY,
    }
}

/// Returns the universe time of the first frame of a new game at `base` on `planet`: the sky moves the
/// clock from 0 to the phase a new game asks for, as `render::sun::track` does.
fn first_frame(planet: &PlanetSpec, base: &Base) -> f64 {
    let now = observer(planet, base.wake, 0.0).phase();
    catch_up(now, START_PHASE, planet)
}

/// The home of seed 7 is locked to its star, and its base used to stand on the night side, the star
/// 60 degrees under the horizon for ever. The base and the place a new game lands at now stand on the
/// day side, the star at [`LIT`] or higher at any time.
#[test]
fn the_base_of_a_locked_home_stands_on_its_day_side() {
    let planet = home(7);
    assert!(planet.locked(), "the home of seed 7 no longer locks");
    let (base, landing) = founded(&planet);
    let lit = LIT.to_radians().sin() - SLACK;
    for time in [0.0, 5.0e4, 3.0e6, 1.0e9] {
        let star = sun(&planet, time);
        let (over_base, over_landing) = (star.dot(base.up), star.dot(landing.normalize()));
        println!("at {time} s the star stands at {over_base:.3} over the base");
        assert!(over_base >= lit, "base: {over_base} at {time} s");
        assert!(over_landing >= lit, "landing: {over_landing} at {time} s");
    }
}

/// The same planet made to turn takes its base by the same rule: the star climbs to [`LIT`] over the
/// base and over the landing in the course of a day, and the base lies at a mild latitude.
#[test]
fn the_base_of_a_turning_planet_sees_the_star_climb() {
    let planet = turning();
    let (base, landing) = founded(&planet);
    let (day, lit) = (Daystar::of(&planet), LIT.to_radians().sin() - SLACK);
    assert!(day.peak(base.up) >= lit, "base: {}", day.peak(base.up));
    assert!(day.peak(landing.normalize()) >= lit, "landing");
    let degrees = base.up.y.asin().to_degrees().abs();
    assert!(degrees < 50.0, "base at {degrees} degrees");
}

/// A new game opens in daylight: on its first frame the star stands above the horizon of the base, on the
/// homes of seeds 0 to 8 and on a home made to turn, where the first frame is the early morning.
#[test]
fn a_new_game_opens_with_the_star_in_the_sky() {
    let planets = (0..=8).map(home).chain([turning()]);
    for planet in planets {
        let (base, _) = founded(&planet);
        let time = first_frame(&planet, &base);
        let height = sun(&planet, time).dot(base.wake.normalize());
        println!(
            "{} locked {}: first frame at {time:.0} s, star at {height:.2}",
            planet.name,
            planet.locked()
        );
        assert!(height > 0.0, "{}: night at the first frame", planet.name);
        if !planet.locked() {
            let phase = observer(&planet, base.wake, time).phase();
            assert!((phase - START_PHASE).abs() < 2e-3, "phase {phase}");
            assert!(height > 0.25, "{}: the star at {height}", planet.name);
        }
    }
}
