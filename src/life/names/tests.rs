//! Tests of species names and epithets.

use super::*;
use crate::life::{Glow, GlowUse, Life};

/// Returns the star and life of the first living body around the home sector whose web holds a
/// species that `wanted` accepts.
fn living(wanted: fn(&crate::life::Species) -> bool) -> (StarSpec, Life) {
    let lives = crate::life::web::tests::lives(60).into_iter();
    let found = lives
        .map(|(star, _, life)| (star, life))
        .find(|(_, l)| l.species.iter().any(wanted));
    found.expect("a living body holds the species wanted")
}

/// The same species gets the same name, sectors of one cube share a region, and far ones do not.
#[test]
fn names_are_seeded_and_regional() {
    let (star, life) = living(|_| true);
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

/// Returns true when `species` is a hunter that does not fly.
fn ground_hunter(species: &crate::life::Species) -> bool {
    species.role.guild == Guild::Hunter && species.role.habitat != Habitat::Air
}

/// The epithet names a glow first, and a ground hunter as a hunter.
#[test]
fn epithet_reads_plainly() {
    let (_, mut life) = living(ground_hunter);
    let hunter = life.species.iter_mut().find(|s| ground_hunter(s));
    let hunter = hunter.expect("the web has a ground hunter");
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
