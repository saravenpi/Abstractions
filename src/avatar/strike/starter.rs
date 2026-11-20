//! Starter tools, in dev builds only: a key puts a spear, then a club, then nothing in the hotbar of
//! the pack, each built by the solver from what the planet underfoot grows and held from there.

use bevy::prelude::*;

use super::stock::{hardest, stock};
use crate::craft::build::{Intent, Place, mass};
use crate::craft::pack::{SLOTS, Stack};
use crate::craft::{Burden, Pack};
use crate::world::Terrain;

/// The key that hands over the next starter tool.
pub const KEY: KeyCode = KeyCode::F8;

/// Stows `stock` the pack does not hold yet, so a second press does not pile it up.
fn stow(pack: &mut Pack, stock: &[crate::craft::build::Stock]) {
    for found in stock {
        if pack.stacks.iter().all(|s| s.stuff != found.stuff) {
            let stack = Stack {
                stuff: found.stuff,
                material: found.material,
                mass: found.mass,
            };
            pack.stow(stack, f32::INFINITY);
        }
    }
}

/// Puts the next starter tool in the pack and holds it: a spear, a club, then an empty slot, round
/// again.
pub fn give(
    keys: Res<ButtonInput<KeyCode>>,
    (terrain, burden): (Res<Terrain>, Res<Burden>),
    mut pack: ResMut<Pack>,
    mut turn: Local<u8>,
) {
    if !keys.just_pressed(KEY) {
        return;
    }
    *turn = (*turn + 1) % 3;
    let intent = [None, Some(Intent::Throw), Some(Intent::Crush)][usize::from(*turn)];
    let globe = terrain.globe();
    let (Some(intent), Some(life)) = (intent, globe.life()) else {
        let empty = pack.bar.slots.iter().position(Option::is_none);
        pack.bar.select(empty.unwrap_or(SLOTS));
        return;
    };
    let found = stock(life, globe.record(), globe.seed());
    let place = Place::on(globe.gravity(), globe.record());
    let Some(best) = hardest(intent, &found, &place) else {
        return;
    };
    stow(&mut pack, &found);
    let Some(id) = pack.make(&best, &found) else {
        return;
    };
    let slot = pack.bar.slots.iter().position(|s| *s == Some(id));
    pack.bar.select(slot.unwrap_or(SLOTS));
    info!(
        "starter {intent:?} {id}: {:.2} kg, carrying {:.1} of {:.1} kg",
        mass(&best.build.pieces()),
        pack.mass(),
        burden.capacity
    );
}
