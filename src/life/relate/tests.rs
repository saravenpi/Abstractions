//! Tests of the relationships other than eating.

use super::*;
use crate::life::Life;
use crate::universe::Universe;

/// Returns the life of Kodya in universe `seed`.
fn kodya(seed: u64) -> Life {
    let universe = Universe::new(seed);
    let star = universe.system(universe.home().star).star;
    crate::life::web::assemble(&universe.planet(universe.home()), &star).expect("Kodya lives")
}

/// Returns true when `relation` joins species of the right kinds in `life`.
fn fits(life: &Life, relation: Relation) -> bool {
    let guild = |i: u16| life.species[usize::from(i)].role.guild;
    match relation {
        Relation::Pollinates { animal, plant } => {
            guild(animal) == Guild::Nectar && guild(plant) == Guild::Producer
        }
        Relation::Sows { animal, plant } => {
            guild(animal) == Guild::Frugivore && guild(plant) == Guild::Producer
        }
        Relation::Rides { rider, host } => {
            rider != host && matches!(guild(host), Guild::Grazer | Guild::Browser)
        }
        Relation::Mimics { mimic, model } => {
            mimic != model && !toxic(&life.species[usize::from(mimic)])
        }
        Relation::Builds { species, .. } => guild(species) != Guild::Producer,
    }
}

/// Relations join species of the right kinds, and Kodya always has a keystone.
#[test]
fn relations_are_consistent() {
    for seed in 0..30 {
        let life = kodya(seed);
        assert!(
            life.relations.iter().all(|r| fits(&life, *r)),
            "seed {seed}"
        );
        assert!(life.keystone.is_some(), "seed {seed}: no keystone");
    }
}

/// A mimic wears the colours of its model's warning tissue.
#[test]
fn mimics_copy_their_model() {
    let mimicry = (0..60).map(kodya).find_map(|life| {
        let pair = life.relations.iter().find_map(|r| match *r {
            Relation::Mimics { mimic, model } => Some((mimic, model)),
            _ => None,
        });
        pair.map(|pair| (life, pair))
    });
    let (life, (mimic, model)) = mimicry.expect("some Kodya has a mimic");
    let model = &life.species[usize::from(model)];
    let warning = model.tissues.iter().find(|t| t.reaction != Reaction::Inert);
    assert_eq!(
        life.species[usize::from(mimic)].tissues[0].colour,
        warning.expect("a toxic model").colour
    );
}
