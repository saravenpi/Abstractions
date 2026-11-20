//! Picking stones up: the pick key breaks up the rock under the crosshair, within the reach of the
//! robot. The stone is one of the stones of the planet, chosen by the id of the rock, so the same rock
//! always gives the same one. The robot carries off what it can of the rock, and the rock is gone
//! from the world for good: the rest of it is rubble nobody tracks.

mod aim;

use bevy::prelude::*;

use self::aim::Aim;
use super::tune::PICK_KEY;
use super::{Burden, Pack, Stack};
use crate::craft::matter::{Mineral, Stuff};
use crate::space::Location;
use crate::universe::PlanetRecord;
use crate::world::Terrain;
use crate::world::rocks::{Rock, RockId, RockTaken, Taken};

/// Returns the stones that lie on a planet of `record`.
pub fn stones(record: &PlanetRecord) -> Vec<Mineral> {
    Mineral::ALL
        .into_iter()
        .filter(|m| m.lies_on(record))
        .collect()
}

/// Returns all of `rock` as the stone of the planet of `record` its id picks, or nothing when no
/// stone lies there. Its mass is its volume times the density of the stone.
pub fn broken(record: &PlanetRecord, rock: &Rock) -> Option<Stack> {
    let found = stones(record);
    let stone = *found.get(rock.id.0 as usize % found.len().max(1))?;
    let material = stone.material();
    Some(Stack {
        stuff: Stuff::Mineral(stone),
        material,
        mass: rock.mass(material.density),
    })
}

/// Breaks up `rock`, when there is one, and stows as much of it as `room` kilograms hold. The rock
/// is noted as taken and its id returned when the robot carried off any of it. A rock the robot
/// cannot carry the least of stays where it is.
pub(super) fn take(
    rock: Option<Rock>,
    (record, room): (&PlanetRecord, f32),
    (pack, taken): (&mut Pack, &mut Taken),
) -> Option<RockId> {
    let rock = rock?;
    let stack = broken(record, &rock)?;
    (pack.stow(stack, room) > 0.0).then_some(())?;
    taken.add(rock.id);
    Some(rock.id)
}

/// Breaks up the rock under the crosshair when the pick key goes down, and says so.
pub(crate) fn gather(
    (keys, aim): (Res<ButtonInput<KeyCode>>, Aim),
    (terrain, burden, location): (Res<Terrain>, Res<Burden>, Res<Location>),
    (mut pack, mut taken): (ResMut<Pack>, ResMut<Taken>),
    mut told: MessageWriter<RockTaken>,
) {
    if !keys.just_pressed(PICK_KEY) {
        return;
    }
    let record = terrain.globe().record();
    let tools = (&mut *pack, &mut *taken);
    if let Some(id) = take(aim.rock(), (record, burden.room()), tools) {
        told.write(RockTaken {
            planet: location.body,
            id,
        });
    }
}
