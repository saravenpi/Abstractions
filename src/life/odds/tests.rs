//! Tests of the odds of life over real systems of a seeded universe.

use super::*;
use crate::universe::{PlanetClass, PlanetSpec, StarSpec, Universe};

/// Returns `count` systems, each a star with its planets and their moons, from the home sector
/// and its 26 neighbours in universes 1, 2 and so on.
pub(in crate::life) fn systems(count: usize) -> Vec<(StarSpec, Vec<PlanetSpec>)> {
    let mut out = Vec::with_capacity(count);
    for seed in 1.. {
        let universe = Universe::new(seed);
        let home = universe.home().star;
        for step in 0..27 {
            let sector =
                home.sector + bevy::math::IVec3::new(step % 3 - 1, step / 3 % 3 - 1, step / 9 - 1);
            for star in universe.stars_in_sector(home.galaxy, sector) {
                let system = universe.system(star.id);
                let ringed = system.planets.iter().filter(|p| p.moons > 0);
                let moons = ringed.flat_map(|p| universe.moons(p.id));
                let bodies = system.planets.iter().cloned().chain(moons).collect();
                out.push((system.star, bodies));
            }
        }
        if out.len() >= count {
            break;
        }
    }
    out.truncate(count);
    out
}

/// Returns the rung of `planet` around `star`.
fn of(planet: &PlanetSpec, star: &StarSpec) -> Option<Rung> {
    rung(&World::of(planet, star), planet.seed)
}

/// Between 15 and 95 percent of systems near home hold a living world. Prints the share.
///
/// The universe puts a liquid water world in about 84 percent of the systems near home, and an
/// Earth twin lives in more than 90 percent of its universes, so the share sits near 0.88.
/// The planet wave will bring it to 20 to 40 percent by making water worlds rarer.
#[test]
fn living_systems_are_a_sane_share() {
    let systems = systems(400);
    let living = systems
        .iter()
        .filter(|(star, bodies)| bodies.iter().any(|p| of(p, star).is_some()))
        .count();
    let share = living as f32 / systems.len() as f32;
    println!("living systems: {share:.3} of {}", systems.len());
    assert!((0.15..0.95).contains(&share), "living share {share}");
}

/// Barren worlds and gas giants never live, and lava worlds rarely do, at most 5 percent of them,
/// and only as mats. Prints the share of lava worlds that live.
#[test]
fn harsh_worlds_stay_low() {
    let (mut lava, mut living) = (0, 0);
    for (star, bodies) in systems(600) {
        for planet in bodies {
            let rung = of(&planet, &star);
            match planet.record.class {
                PlanetClass::Barren | PlanetClass::GasGiant => assert_eq!(rung, None),
                PlanetClass::Lava => {
                    assert!(rung.is_none_or(|r| r == Rung::Mats));
                    lava += 1;
                    living += usize::from(rung.is_some());
                }
                _ => {}
            }
        }
    }
    let share = living as f32 / lava as f32;
    println!("lava worlds living: {living} of {lava}, {share:.3}");
    assert!(share <= 0.05, "lava living share {share}");
}

/// The same world and seed always give the same rung, and every rung shows up somewhere.
#[test]
fn rung_is_seeded_and_spread() {
    let home = World {
        gravity: 22.0,
        pressure: 1.0,
        temperature: 288.0,
        flux: 1.0,
        radius: 6000.0,
        ocean: 0.6,
        star: 5780.0,
        tilt: 0.0,
        liquids: [0.6, 0.0375, 0.0375, 0.6],
    };
    let rungs: Vec<_> = (0..400).filter_map(|seed| rung(&home, seed)).collect();
    assert!(LADDER.iter().all(|r| rungs.contains(r)));
    assert!((0..50).all(|seed| rung(&home, seed) == rung(&home, seed)));
}

/// The home planet, an Earth twin, rolls by the same rules as every planet and lives in more than
/// 90 percent of 200 universes. Prints its suitability and how many reach each rung.
#[test]
fn home_lives_by_the_same_odds() {
    let mut rungs = [0; 6];
    for seed in 0..200 {
        let universe = Universe::new(seed);
        let home = universe.home();
        let (planet, star) = (universe.planet(home), universe.system(home.star).star);
        let rung = of(&planet, &star);
        assert_eq!(crate::life::of(&planet, &star).map(|l| l.rung), rung);
        rungs[rung.map_or(0, |r| 1 + LADDER.iter().position(|l| *l == r).unwrap_or(0))] += 1;
        if seed == 0 {
            let world = World::of(&planet, &star);
            println!("{}: {world:?} suits {:?}", planet.name, suitability(&world));
        }
    }
    println!("of 200 homes: dead, mats, soft, shelled, skeletal, old = {rungs:?}");
    assert!(rungs[0] < 20, "{} of 200 homes are dead", rungs[0]);
}
