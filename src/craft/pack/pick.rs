//! Picking stones up: the pick key breaks a piece off the rock under the crosshair, within the reach of
//! the robot. The stone is one of the stones of the planet, chosen by the id of the rock, so the same
//! rock always gives the same one. A pick takes at most [`HANDFUL`] kilograms and what the robot can
//! still carry; the rock keeps the rest, shrinks as it is quarried and is gone when its mass is used up.

mod aim;

use bevy::prelude::*;

use self::aim::Aim;
use super::tune::{HANDFUL, PICK_KEY};
use super::{Burden, Pack, Stack};
use crate::craft::matter::{Mineral, Stuff};
use crate::player::{Interact, Verb};
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

/// Returns the stone of the planet of `record` that the rock `id` is made of, or nothing when no
/// stone lies there.
pub fn stone_of(record: &PlanetRecord, id: RockId) -> Option<Mineral> {
    let found = stones(record);
    found.get(id.0 as usize % found.len().max(1)).copied()
}

/// Breaks a piece off `rock`, when there is one, and stows it: at most [`HANDFUL`] kilograms, at most
/// what is left of the rock, at most `room` kilograms. The piece is noted as taken from the rock, and
/// its mass and the rock's id are returned when the robot carried off any. A robot with no room
/// leaves the rock as it is.
pub(super) fn take(
    rock: Option<Rock>,
    (record, room): (&PlanetRecord, f32),
    (pack, taken): (&mut Pack, &mut Taken),
) -> Option<(RockId, f32)> {
    let rock = rock?;
    let stone = stone_of(record, rock.id)?;
    let material = stone.material();
    let whole = taken
        .whole(rock.id)
        .unwrap_or_else(|| rock.mass(material.density));
    let piece = HANDFUL.min(whole - taken.kilograms(rock.id));
    let stack = Stack {
        stuff: Stuff::Mineral(stone),
        material,
        mass: piece,
    };
    let kept = pack.stow(stack, room);
    (kept > 0.0).then_some(())?;
    taken.quarry(rock.id, kept, whole);
    Some((rock.id, kept))
}

/// Offers to break a piece off the rock under the crosshair, and breaks it off when the pick key goes
/// down, and says so.
pub(crate) fn gather(
    (keys, aim): (Res<ButtonInput<KeyCode>>, Aim),
    (terrain, burden, location): (Res<Terrain>, Res<Burden>, Res<Location>),
    (mut pack, mut taken, mut interact): (ResMut<Pack>, ResMut<Taken>, ResMut<Interact>),
    mut told: MessageWriter<RockTaken>,
) {
    let record = terrain.globe().record();
    let rock = aim
        .rock()
        .filter(|rock| stone_of(record, rock.id).is_some());
    if rock.is_none() {
        return;
    }
    interact.offer(Verb::Stone, "");
    if !keys.just_pressed(PICK_KEY) {
        return;
    }
    let tools = (&mut *pack, &mut *taken);
    if let Some((id, kilograms)) = take(rock, (record, burden.room()), tools) {
        told.write(RockTaken {
            planet: location.body,
            id,
            kilograms,
        });
    }
}
