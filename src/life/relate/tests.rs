//! Tests of the relationships other than eating.

use super::*;
use crate::life::Life;

/// Returns the life of the living bodies of the first `count` systems around the home sector.
fn lives(count: usize) -> impl Iterator<Item = Life> {
    crate::life::web::tests::lives(count)
        .into_iter()
        .map(|(_, _, life)| life)
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

/// Relations join species of the right kinds, and every web with a hunter or an ambusher has a keystone.
#[test]
fn relations_are_consistent() {
    for life in lives(100) {
        assert!(life.relations.iter().all(|r| fits(&life, *r)));
        let predator =
            |s: &crate::life::Species| matches!(s.role.guild, Guild::Hunter | Guild::Ambusher);
        assert!(life.keystone.is_some() || !life.species.iter().any(predator));
    }
}

/// A mimic wears the colours of its model's warning tissue.
#[test]
fn mimics_copy_their_model() {
    let mimicry = lives(200).find_map(|life| {
        let pair = life.relations.iter().find_map(|r| match *r {
            Relation::Mimics { mimic, model } => Some((mimic, model)),
            _ => None,
        });
        pair.map(|pair| (life, pair))
    });
    let (life, (mimic, model)) = mimicry.expect("some web has a mimic");
    let model = &life.species[usize::from(model)];
    let warning = model.tissues.iter().find(|t| t.reaction != Reaction::Inert);
    assert_eq!(
        life.species[usize::from(mimic)].tissues[0].colour,
        warning.expect("a toxic model").colour
    );
}
