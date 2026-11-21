//! A kill: the animal goes into the kill ledger with where and when, leaves a carcass that every
//! player sees, and falls where it stood.

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

use super::Dead;
use super::slump::Slump;
use super::tune::LEAN;
use crate::craft::matter::Grade;
use crate::fauna::beast::Beast;
use crate::fauna::herds::Herds;
use crate::fauna::want::{Lead, carcass_key};
use crate::life::{Species, hunt};
use crate::world::Terrain;

/// What was taken from one carcass and how cleanly its animal was killed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Carved {
    /// Identity of the carcass.
    pub key: u64,
    /// The parts taken, a bit for each part index.
    pub taken: u64,
    /// How cleanly it was killed.
    pub kill: Grade,
}

/// Returns the fall of `beast` of `species` at its place, lying on the side its key chooses.
fn fall(beast: &Beast, species: &Species, (key, ground): (u64, f32)) -> Slump {
    let root = &beast.motion.root;
    let ahead = (root.turn * Vec3::NEG_Z)
        .with_y(0.0)
        .normalize_or(Vec3::NEG_Z);
    let body = &species.body;
    let pivot = Vec3::new(root.at.x, root.place.y.max(ground), root.at.z);
    Slump {
        pivot,
        axis: if key & 1 == 0 { ahead } else { -ahead },
        side: (0.3 * body.height).max(0.05),
        length: 2.0 * body.height,
        angle: LEAN.0,
        rate: LEAN.1,
        rest: std::f32::consts::FRAC_PI_2,
        air: pivot.y - ground,
        fell: 0.0,
        sink: 0.0,
    }
}

/// Records the death of the animal `entity`, whose `beast` of `species` was killed with `kill`: in
/// the ledger of its herd at universe time `now`, as a carcass at the place it stood, and as a body
/// that falls and stays.
pub fn slay(
    commands: &mut Commands,
    (entity, beast): (Entity, &mut Beast),
    (herds, now): (&mut Herds, f64),
    (terrain, kill): (&Terrain, Grade),
) {
    let Lead::Herd { id, slot } = beast.lead else {
        return;
    };
    let Some(life) = terrain.globe().life() else {
        return;
    };
    let Some(species) = life.species.get(usize::from(beast.index)) else {
        return;
    };
    let at = beast.spot.planet(beast.motion.root.at).normalize();
    let carcass = hunt::slain(life, (id, slot, beast.index), at, now);
    let key = carcass_key(&carcass);
    let ground = beast.spot.ground(terrain, beast.motion.root.at.xz());
    herds.ledger.kill(id, slot, now);
    herds.carved.push(Carved {
        key,
        taken: 0,
        kill,
    });
    herds.slain.push(carcass);
    commands
        .entity(entity)
        .insert((Dead, fall(beast, species, (key, ground))));
    beast.lead = Lead::Carcass { key, at };
}
