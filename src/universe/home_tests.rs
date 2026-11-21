//! Home tests: the home planet is chosen by the same rule for every seed, never made by hand.

use std::collections::HashSet;
use std::time::Instant;

use super::home::{HOME_GALAXY, SEARCH, homely};
use super::range_tests::sample;
use super::*;
use crate::life;

/// Seeds the home tests look at.
const SEEDS: u64 = 40;

/// Returns the home planet and its star for a seed.
fn home(seed: u64) -> (PlanetSpec, StarSpec) {
    let u = Universe::new(seed);
    (u.planet(u.home()), u.system(u.home().star).star)
}

/// Returns true when a landable planet of `star` is homely under the normal odds.
fn welcomes(u: &Universe, star: &StarSpec) -> bool {
    let planets = u.system(star.id).planets;
    let fits = |p: &PlanetSpec| {
        let world = life::World::of(p, star);
        let rung = life::odds::rung(&world, p.seed);
        landable(p) && rung.is_some_and(|rung| homely(p, rung, &world))
    };
    planets.iter().any(fits)
}

/// The home is a solid planet a robot can land on, and it holds life under the same odds as
/// every other planet, with seas, plants and herds and a mild mean temperature. A seed whose
/// nearest systems hold no such planet may fall back on a less fitting one: that is printed,
/// and stays rare.
#[test]
fn home_is_landable_and_alive() {
    let mut dead = Vec::new();
    for seed in 0..SEEDS {
        let (planet, star) = home(seed);
        assert!(landable(&planet) && !planet.id.is_moon(), "seed {seed}");
        assert!(planet.record.class != PlanetClass::GasGiant, "seed {seed}");
        let world = life::World::of(&planet, &star);
        let rung = life::odds::rung(&world, planet.seed);
        if !rung.is_some_and(|rung| homely(&planet, rung, &world)) {
            dead.push(seed);
        }
    }
    println!("seeds whose home is not homely: {dead:?}");
    assert!(dead.is_empty(), "{dead:?}");
}

/// Under the rule of 0.11.0, the rule of the worlds saved before the home had to be Earth-like,
/// no star nearer to the Sun-like point of the home galaxy than the home star has a homely
/// landable planet: the home is the nearest one.
#[test]
fn home_is_the_nearest_homely_planet() {
    for seed in 0..12 {
        let u = Universe::under(seed, 0);
        let (planet, star) = (u.planet(u.home()), u.system(u.home().star).star);
        let world = life::World::of(&planet, &star);
        let rung = life::odds::rung(&world, planet.seed);
        if !rung.is_some_and(|rung| homely(&planet, rung, &world)) {
            continue;
        }
        let shape = galaxy::shape(seed, HOME_GALAXY).expect("the home galaxy exists");
        let from = shells::sun_point(&shape);
        let out = star.pos.distance(from) / LY * (1.0 + 1e-12);
        let nearer = u.stars_near(HOME_GALAXY, from, out);
        assert_eq!(nearer.last().map(|s| s.id), Some(star.id), "seed {seed}");
        assert!(nearer.len() <= SEARCH, "seed {seed}: {}", nearer.len());
        let inside = &nearer[..nearer.len() - 1];
        assert!(inside.iter().all(|s| !welcomes(&u, s)), "seed {seed}");
    }
}

/// The home planet and its star carry generated names, and nothing about the home is fixed:
/// names, sizes, pulls, days and stars differ from seed to seed.
#[test]
fn nothing_about_the_home_is_fixed() {
    let homes: Vec<(PlanetSpec, StarSpec)> = (0..SEEDS).map(home).collect();
    let distinct = |key: &dyn Fn(&(PlanetSpec, StarSpec)) -> u64| {
        homes.iter().map(key).collect::<HashSet<u64>>().len()
    };
    for (planet, star) in &homes {
        assert_eq!(planet.name, names::planet(&star.name, planet.id.index));
        assert!(!star.name.is_empty());
    }
    let names: HashSet<&str> = homes.iter().map(|(p, _)| p.name.as_str()).collect();
    assert!(names.len() >= 38, "{names:?}");
    assert!(distinct(&|(p, _)| p.radius.to_bits()) >= 38);
    assert!(distinct(&|(p, _)| p.gravity.to_bits()) >= 38);
    assert!(distinct(&|(p, _)| p.day_seconds.to_bits()) >= 38);
    assert!(distinct(&|(_, s)| s.temperature.to_bits()) >= 38);
    assert!(distinct(&|(p, _)| u64::from(p.id.index)) >= 2);
    assert!(distinct(&|(p, _)| u64::from(p.moons)) >= 2);
}

/// Life lands in about 20 to 40 percent of systems, counting planets and moons: wet worlds are
/// the exception. The share is printed, with the share of systems that hold liquid water.
#[test]
fn life_is_in_a_minority_of_systems() {
    let (mut systems, mut alive, mut wet) = (0.0, 0.0, 0.0);
    for (u, star) in sample() {
        let planets = u.system(star.id).planets;
        let moons = planets.iter().flat_map(|p| moon::of(&star, p));
        let bodies: Vec<PlanetSpec> = planets.iter().cloned().chain(moons).collect();
        systems += 1.0;
        alive += f64::from(u8::from(
            bodies.iter().any(|b| life::of(b, &star).is_some()),
        ));
        wet += f64::from(u8::from(bodies.iter().any(|b| b.record.liquids()[0] > 0.0)));
    }
    let (alive, wet) = (100.0 * alive / systems, 100.0 * wet / systems);
    println!("systems with life {alive:.1} %, with liquid water {wet:.1} %, of {systems}");
    assert!((17.0..=43.0).contains(&alive), "{alive}");
    assert!(wet < 50.0, "{wet}");
}

/// Building a universe finds its home in a few milliseconds: 100 ms still catches a search
/// that walks the whole galaxy.
#[test]
fn the_home_search_is_cheap() {
    let run = |seed: u64| {
        let start = Instant::now();
        let home = Universe::new(seed).home();
        (start.elapsed(), home)
    };
    let worst = (0..20).map(|seed| (0..3).map(|_| run(seed).0).min());
    let worst = worst.flatten().max().expect("twenty seeds");
    println!("slowest home search of 20 seeds: {worst:?}");
    assert!(worst.as_millis() < 100, "{worst:?}");
}
