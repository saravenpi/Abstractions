//! Tests of the assembled life of real planets of a seeded universe.

use super::*;
use crate::life::guild::FLIGHT_AIR;
use crate::life::{Guild, Habitat, Plan};
use crate::universe::Universe;

/// Returns the life of every living body of the first `count` systems around the home sector.
fn lives(count: usize) -> Vec<(PlanetSpec, Life)> {
    let systems = crate::life::odds::tests::systems(count);
    let bodies = systems
        .into_iter()
        .flat_map(|(star, bodies)| bodies.into_iter().map(move |p| (star.clone(), p)));
    bodies
        .filter_map(|(star, p)| assemble(&p, &star).map(|life| (p, life)))
        .collect()
}

/// Returns the home planet and star of universe `seed`.
fn kodya(seed: u64) -> (PlanetSpec, StarSpec) {
    let universe = Universe::new(seed);
    let home = universe.home();
    (universe.planet(home), universe.system(home.star).star)
}

/// The same planet always gives the same life.
#[test]
fn same_seed_same_life() {
    let (planet, star) = kodya(3);
    assert_eq!(assemble(&planet, &star), assemble(&planet, &star));
    for (planet, life) in lives(40).into_iter().take(5) {
        let star = Universe::new(1).system(planet.id.star).star;
        assert_eq!(assemble(&planet, &star), Some(life));
    }
}

/// Checks that species `s` sits at index `i`, has one to three tissues, and eats as its guild says:
/// a producer eats nothing and has no body, a consumer eats something.
fn fits(i: usize, s: &Species) {
    assert_eq!(usize::from(s.id.index), i);
    assert!((1..=3).contains(&s.tissues.len()));
    match s.role.guild {
        Guild::Producer => {
            assert!(s.prey.is_empty() && s.body.parts.is_empty() && s.plan == Plan::Colonial)
        }
        _ => assert!(!s.prey.is_empty(), "{} eats nothing", s.name),
    }
}

/// Checks that `life` of `planet` has producers first, consumers with prey, and a connectance in range.
fn holds_together(planet: &PlanetSpec, life: &Life) {
    let species = &life.species;
    let producers = species
        .iter()
        .take_while(|s| s.role.guild == Guild::Producer)
        .count();
    assert!(producers >= 1, "{}: no producer", planet.name);
    assert!(
        species
            .iter()
            .skip(producers)
            .all(|s| s.role.guild != Guild::Producer)
    );
    species.iter().enumerate().for_each(|(i, s)| fits(i, s));
    if species.len() > producers {
        let links: usize = species.iter().map(|s| s.prey.len()).sum();
        let connectance = links as f32 / (species.len() * species.len()) as f32;
        assert!(
            (0.04..0.3).contains(&connectance),
            "{}: connectance {connectance}",
            planet.name
        );
        assert!((12..=40).contains(&species.len()));
    }
}

/// Every living planet has producers first, consumers with prey, and a connectance in range.
#[test]
fn webs_hold_together() {
    let lives = lives(300);
    assert!(lives.len() > 50, "only {} living bodies", lives.len());
    for (planet, life) in &lives {
        holds_together(planet, life);
    }
}

/// Swimmers live only where there is a liquid, flyers only in thick enough air, and mats never move.
#[test]
fn habitats_fit_the_planet() {
    for (planet, life) in lives(300) {
        let record = &planet.record;
        for s in &life.species {
            assert!(s.role.habitat != Habitat::Water || record.ocean > 0.0);
            assert!(s.role.habitat != Habitat::Air || record.pressure >= f64::from(FLIGHT_AIR));
        }
        if life.rung == Rung::Mats {
            assert!(
                life.species.iter().all(|s| s.role.guild == Guild::Producer)
                    && life.species.len() <= 3
            );
        }
    }
}

/// Kodya lives at the skeletal rung with a ground grazer that lives in herds, and a hunter, for every seed.
#[test]
fn kodya_has_a_herd_and_a_hunter() {
    for seed in 0..40 {
        let (planet, star) = kodya(seed);
        let life = assemble(&planet, &star).expect("Kodya lives");
        assert_eq!(life.rung, Rung::Skeletal);
        let herd = life.species.iter().any(|s| {
            let r = &s.role;
            r.guild == Guild::Grazer
                && r.habitat == Habitat::Ground
                && s.plan == Plan::Skeletal
                && s.history.group_max >= 4
        });
        assert!(herd, "seed {seed}: no grazer herd");
        assert!(
            life.species.iter().any(|s| s.role.guild == Guild::Hunter),
            "seed {seed}: no hunter"
        );
    }
}
