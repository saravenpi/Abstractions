//! The spears this robot threw that stuck in an animal, and the ones it drew out, sent to the
//! server.

use abstractions_net::wire::{Act, ClientMsg, Spear as Stuck, SpearKey};
use bevy::prelude::*;

use super::Hunted;
use super::blows::key_of;
use super::item;
use crate::avatar::strike::{Rest, Spear};
use crate::fauna::Creature;
use crate::net::link::Link;
use crate::space::Location;

/// Tells the server of each spear that just stuck in an animal, and numbers it.
pub fn sticks(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    location: Option<Res<Location>>,
    spears: Query<(Entity, &Spear, &Transform, &ChildOf), Changed<Spear>>,
    animals: Query<&Creature>,
) {
    let (Some(location), Some(owner)) = (location, link.you) else {
        return;
    };
    for (entity, spear, at, of) in &spears {
        let Ok(creature) = animals.get(of.parent()) else {
            continue;
        };
        let bytes = item::pack(&spear.item.recipe);
        let fresh = !hunted.planted.contains_key(&entity);
        let (true, true, Some(item)) = (spear.rest == Rest::Stuck, fresh, bytes) else {
            continue;
        };
        hunted.tag += 1;
        let key = SpearKey {
            owner,
            tag: hunted.tag,
        };
        hunted.planted.insert(entity, (key, of.parent()));
        let along = at.rotation * Vec3::Y;
        link.send(ClientMsg::Hunt(Act::Stick(Stuck {
            key,
            animal: key_of(&location, creature),
            at: at.translation.to_array(),
            dir: along.to_array(),
            item,
        })));
    }
}

/// Tells the server of each spear drawn out of an animal that still stands.
pub fn pulls(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    mut gone: RemovedComponents<Spear>,
    animals: Query<(), With<Creature>>,
) {
    for entity in gone.read() {
        let Some((key, animal)) = hunted.planted.remove(&entity) else {
            continue;
        };
        if animals.contains(animal) {
            link.send(ClientMsg::Hunt(Act::Pull(key)));
        }
    }
}
