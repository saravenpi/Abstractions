//! Harm: a blow that lands on an animal hurts it by its energy; a blow that kills marks the death in
//! the kill ledger and keeps later blows from landing on the body.

use bevy::prelude::*;

use crate::avatar::strike::Hit;
use crate::fauna::beast::Beast;
use crate::fauna::herds::Herds;
use crate::fauna::want::Lead;
use crate::space::Clock;

/// An animal that was killed: blows and spears pass through it while it fades out.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Dead;

/// Takes the energy of each blow that landed from the health of the animal struck. A kill goes into
/// the ledger of its herd at the universe time and marks the animal [`Dead`].
pub fn harm(
    mut commands: Commands,
    mut hits: MessageReader<Hit>,
    mut herds: ResMut<Herds>,
    clock: Res<Clock>,
    mut beasts: Query<&mut Beast, Without<Dead>>,
) {
    for hit in hits.read() {
        let Ok(mut beast) = beasts.get_mut(hit.victim) else {
            continue;
        };
        let Lead::Herd { id, slot } = beast.lead else {
            continue;
        };
        if beast.wits.health.hurt(hit.blow.energy) {
            herds.ledger.kill(id, slot, clock.seconds);
            commands.entity(hit.victim).insert(Dead);
        }
    }
}
