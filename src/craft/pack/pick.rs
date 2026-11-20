//! Picking stones up: a key press on foot stows a handful of a stone that lies on the planet
//! underfoot, one kind after the other. The world has no rock marker to look for yet, so any ground
//! gives them, at most once every [`PICK_GAP`] seconds.

use bevy::prelude::*;

use super::tune::{HANDFUL, PICK_GAP, PICK_KEY};
use super::{Burden, Pack, Stack};
use crate::craft::matter::{Mineral, Stuff};
use crate::universe::PlanetRecord;
use crate::world::Terrain;

/// Returns the stones that lie on a planet of `record`.
pub fn stones(record: &PlanetRecord) -> Vec<Mineral> {
    Mineral::ALL
        .into_iter()
        .filter(|m| m.lies_on(record))
        .collect()
}

/// Returns the handful of the stone `turn` of the stones of `record`, going round, or nothing when
/// none lies there.
pub fn handful(record: &PlanetRecord, turn: usize) -> Option<Stack> {
    let found = stones(record);
    let stone = *found.get(turn % found.len().max(1))?;
    Some(Stack {
        stuff: Stuff::Mineral(stone),
        material: stone.material(),
        mass: HANDFUL,
    })
}

/// Stows a handful of the next stone of the planet underfoot when the pick key goes down, as far
/// as the robot can carry it.
pub(crate) fn gather(
    (keys, time): (Res<ButtonInput<KeyCode>>, Res<Time>),
    (terrain, burden): (Res<Terrain>, Res<Burden>),
    mut pack: ResMut<Pack>,
    mut last: Local<(f32, usize)>,
) {
    let now = time.elapsed_secs();
    if !keys.just_pressed(PICK_KEY) || now - last.0 < PICK_GAP {
        return;
    }
    let Some(stack) = handful(terrain.globe().record(), last.1) else {
        return;
    };
    *last = (now, last.1 + 1);
    pack.stow(stack, burden.room());
}
