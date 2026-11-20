//! Harm: a blow that lands on an animal hurts it by its energy; a blow that kills marks the death in
//! the kill ledger, keeps later blows from landing on the body and, until carcasses can be cut,
//! hands its largest parts to the harvest.

use bevy::prelude::*;

use crate::avatar::strike::Hit;
use crate::craft::build::Intent;
use crate::craft::harvest::{Cut, Harvest};
use crate::craft::matter::Grade;
use crate::fauna::beast::Beast;
use crate::fauna::herds::Herds;
use crate::fauna::want::Lead;
use crate::life::Species;
use crate::space::Clock;
use crate::world::Terrain;

/// Parts of a kill the temporary harvest takes, the largest first.
const MAIN: usize = 3;

/// An animal that was killed: blows and spears pass through it while it fades out.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Dead;

/// The blows an animal took, which grade its tissue when it dies.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Wounds {
    /// Blows landed.
    pub hits: u8,
    /// True when one of them crushed.
    pub blunt: bool,
}

/// Returns the indices of the [`MAIN`] largest parts of `species`.
fn main_parts(species: &Species) -> Vec<u8> {
    let volume = |at: usize| {
        let size = species.body.parts[at].size;
        size.x * size.y * size.z
    };
    let mut parts: Vec<usize> = (0..species.body.parts.len()).collect();
    parts.sort_by(|a, b| volume(*b).total_cmp(&volume(*a)));
    parts.into_iter().take(MAIN).map(|at| at as u8).collect()
}

/// Asks the harvest for the main parts of the animal of species `index` killed with `wounds`.
fn reap(terrain: &Terrain, (index, wounds): (u16, Wounds), asks: &mut MessageWriter<Harvest>) {
    let Some(species) = terrain
        .globe()
        .life()
        .and_then(|l| l.species.get(usize::from(index)))
    else {
        return;
    };
    let kill = Grade::of(wounds.hits, wounds.blunt);
    for part in main_parts(species) {
        asks.write(Harvest(Cut {
            species: index,
            part,
            kill,
        }));
    }
}

/// Takes the energy of each blow that landed from the health of the animal struck. A kill goes into
/// the ledger of its herd at the universe time, marks the animal [`Dead`] and asks for its parts.
pub fn harm(
    mut commands: Commands,
    mut hits: MessageReader<Hit>,
    (mut herds, clock, terrain): (ResMut<Herds>, Res<Clock>, Res<Terrain>),
    mut beasts: Query<(&mut Beast, Option<&mut Wounds>), Without<Dead>>,
    mut asks: MessageWriter<Harvest>,
) {
    for hit in hits.read() {
        let Ok((mut beast, wounds)) = beasts.get_mut(hit.victim) else {
            continue;
        };
        let Lead::Herd { id, slot } = beast.lead else {
            continue;
        };
        let blunt = hit.intent == Intent::Crush;
        let now = match wounds {
            Some(mut w) => {
                w.hits = w.hits.saturating_add(1);
                w.blunt |= blunt;
                *w
            }
            None => {
                let first = Wounds { hits: 1, blunt };
                commands.entity(hit.victim).insert(first);
                first
            }
        };
        if beast.wits.health.hurt(hit.blow.energy) {
            herds.ledger.kill(id, slot, clock.seconds);
            commands.entity(hit.victim).insert(Dead);
            reap(&terrain, (beast.index, now), &mut asks);
        }
    }
}
