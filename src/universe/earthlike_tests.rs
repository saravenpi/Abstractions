//! Earth-like home tests: every new world starts on a planet with air, pull and warmth like the
//! Earth's, the nearest such one, and a world of the first rule keeps the home it had.

use super::earthlike::earthlike;
use super::home::HOME_GALAXY;
use super::*;
use crate::life;

/// Seeds the Earth-like tests look at.
const SEEDS: u64 = 200;

/// Returns true when `planet` of `star` is a landable Earth-like home under the normal odds.
fn like(planet: &PlanetSpec, star: &StarSpec) -> bool {
    let world = life::World::of(planet, star);
    let rung = life::odds::rung(&world, planet.seed);
    landable(planet) && rung.is_some_and(|rung| earthlike(planet, rung, &world))
}

/// Returns the home planet and its star of `universe`.
fn home(universe: &Universe) -> (PlanetSpec, StarSpec) {
    let id = universe.home();
    (universe.planet(id), universe.system(id.star).star)
}

/// The home of every one of 200 seeds is Earth-like: 0.7 to 1.5 bar of air of nitrogen and the
/// oxygen of its life, 0.8 to 1.2 standard gravities, a mean of 5 to 30 C, and homely, with seas,
/// plants and herds. The seeds that fail are printed, with the pressures of the homes and the
/// share of seeds whose home differs from the first rule.
#[test]
fn every_home_is_earthlike() {
    let (mut failed, mut moved, mut pressures) = (Vec::new(), 0, Vec::new());
    for seed in 0..SEEDS {
        let universe = Universe::new(seed);
        let (planet, star) = home(&universe);
        let record = &planet.record;
        pressures.push(record.pressure);
        let inside = |(low, high): (f64, f64), value: f64| (low..=high).contains(&value);
        let banded = inside(PRESSURE, record.pressure)
            && inside(PULL, planet.gravity)
            && inside(TEMPERATE, record.temperature);
        if !(banded && like(&planet, &star)) {
            failed.push((seed, record.pressure, planet.gravity / STANDARD_GRAVITY));
        }
        moved += usize::from(Universe::under(seed, 0).home() != universe.home());
    }
    pressures.sort_by(f64::total_cmp);
    let at = |share: f64| pressures[((pressures.len() - 1) as f64 * share).round() as usize];
    println!(
        "home pressure min {:.2} median {:.2} max {:.2} bar",
        at(0.0),
        at(0.5),
        at(1.0)
    );
    println!("homes that moved from the first rule: {moved} of {SEEDS}");
    println!("homes that are not Earth-like (seed, bar, g): {failed:?}");
    assert!(failed.is_empty(), "{failed:?}");
}

/// No star nearer to the Sun-like point of the home galaxy than the home star has a landable
/// Earth-like planet, and the home star lies within the [`REACH`] of the search.
#[test]
fn home_is_the_nearest_earthlike_planet() {
    for seed in 0..12 {
        let universe = Universe::new(seed);
        let (planet, star) = home(&universe);
        assert!(
            like(&planet, &star),
            "seed {seed}: the home is not Earth-like"
        );
        let shape = galaxy::shape(seed, HOME_GALAXY).expect("the home galaxy exists");
        let from = shells::sun_point(&shape);
        let out = star.pos.distance(from) / LY * (1.0 + 1e-12);
        let nearer = universe.stars_near(HOME_GALAXY, from, out);
        assert_eq!(nearer.last().map(|s| s.id), Some(star.id), "seed {seed}");
        assert!(nearer.len() <= REACH, "seed {seed}: {}", nearer.len());
        let inside = &nearer[..nearer.len() - 1];
        let none = |s: &StarSpec| !universe.system(s.id).planets.iter().any(|p| like(p, s));
        assert!(inside.iter().all(none), "seed {seed}");
    }
}

/// The two rules build the same universe around a different home: the seed, the stars and the
/// planets are the same, and building either twice gives the same home.
#[test]
fn the_rules_differ_only_in_the_home() {
    for seed in 0..8 {
        let (first, now) = (Universe::under(seed, 0), Universe::under(seed, HOME_RULE));
        assert_eq!(first, Universe::under(seed, 0), "seed {seed}");
        assert_eq!(now, Universe::new(seed), "seed {seed}");
        assert_eq!(first.seed(), now.seed());
        for id in [first.home(), now.home()] {
            assert_eq!(first.planet(id), now.planet(id), "seed {seed}");
        }
    }
}
