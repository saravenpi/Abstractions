//! Carcasses that come into view: an animal drawn lying dead is marked [`Dead`], so blows pass
//! through it, and the parts that were taken from it are gone again.

use bevy::prelude::*;

use super::Dead;
use crate::fauna::beast::Beast;
use crate::fauna::herds::Herds;
use crate::fauna::want::Lead;

/// Marks every newly drawn carcass dead and takes from it what was cut from it before.
pub fn raise(
    mut commands: Commands,
    herds: Res<Herds>,
    mut beasts: Query<(Entity, &mut Beast), Added<Beast>>,
) {
    for (entity, mut beast) in &mut beasts {
        let Lead::Carcass { key, .. } = beast.lead else {
            continue;
        };
        commands.entity(entity).insert(Dead);
        let taken = herds
            .carved
            .iter()
            .find(|c| c.key == key)
            .map_or(0, |c| c.taken);
        for index in (0..beast.parts.len().min(64)).filter(|i| taken >> i & 1 == 1) {
            if let Some(part) = beast.parts[index].take() {
                commands.entity(part).despawn();
            }
        }
    }
}
