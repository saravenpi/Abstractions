//! Tests of species names and epithets.

use super::*;
use crate::life::{Glow, GlowUse, Life};
use crate::universe::Universe;

/// Returns the home star and the life of Kodya in universe `seed`.
fn kodya(seed: u64) -> (StarSpec, Life) {
    let universe = Universe::new(seed);
    let star = universe.system(universe.home().star).star;
    let life = crate::life::web::assemble(&universe.planet(universe.home()), &star);
    (star, life.expect("Kodya lives"))
}

/// The same species gets the same name, sectors of one cube share a region, and far ones do not.
#[test]
fn names_are_seeded_and_regional() {
    let (star, life) = kodya(1);
    let mut next = star.clone();
    let x = star.id.sector.x;
    next.id.sector.x = x.div_euclid(REGION) * REGION + (x.rem_euclid(REGION) + 1) % REGION;
    assert_eq!(region(&star), region(&next));
    let mut far = star.clone();
    far.id.sector.x += 40;
    assert_ne!(region(&star), region(&far));
    let first = &life.species[0];
    assert_eq!(name(region(&star), 9, first), name(region(&star), 9, first));
    assert!(
        first
            .name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_uppercase())
    );
}

/// The epithet names a glow first, and a ground hunter as a hunter.
#[test]
fn epithet_reads_plainly() {
    let (_, mut life) = kodya(2);
    let hunter = life
        .species
        .iter_mut()
        .find(|s| s.role.guild == Guild::Hunter && s.role.habitat != Habitat::Air);
    let hunter = hunter.expect("Kodya has a ground hunter");
    hunter.glow = Some(Glow {
        purpose: GlowUse::Lure,
        colour: [1.0; 3],
        strength: 1.0,
    });
    assert_eq!(epithet(hunter), "the glowing hunter");
}

/// The names of the species of a planet are all different, and every one has an epithet.
#[test]
fn names_are_unique() {
    let systems = crate::life::odds::tests::systems(100);
    let bodies = systems
        .iter()
        .flat_map(|(star, bodies)| bodies.iter().map(move |p| (star, p)));
    for life in bodies.filter_map(|(star, p)| crate::life::web::assemble(p, star)) {
        let mut names: Vec<&str> = life.species.iter().map(|s| s.name.as_str()).collect();
        assert!(life.species.iter().all(|s| s.epithet.starts_with("the ")));
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), life.species.len());
    }
}
