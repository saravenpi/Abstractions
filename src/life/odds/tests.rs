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

/// About one system in three holds a living world, within 10 percent.
#[test]
fn one_system_in_three_lives() {
    let systems = systems(2000);
    let living = systems
        .iter()
        .filter(|(star, bodies)| bodies.iter().any(|p| of(p, star).is_some()))
        .count();
    let share = living as f32 / systems.len() as f32;
    println!("living systems: {share:.3}");
    assert!((0.3..0.367).contains(&share), "living share {share}");
}

/// Lava, barren and gas giant worlds never reach skeletal life; the latter two never live at all.
#[test]
fn harsh_worlds_stay_low() {
    for (star, bodies) in systems(600) {
        for planet in bodies {
            let rung = of(&planet, &star);
            match planet.record.class {
                PlanetClass::Barren | PlanetClass::GasGiant => assert_eq!(rung, None),
                PlanetClass::Lava => assert!(rung.is_none_or(|r| r <= Rung::Shelled)),
                _ => {}
            }
        }
    }
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
    };
    let rungs: Vec<_> = (0..400).filter_map(|seed| rung(&home, seed)).collect();
    assert!(LADDER.iter().all(|r| rungs.contains(r)));
    assert!((0..50).all(|seed| rung(&home, seed) == rung(&home, seed)));
}
