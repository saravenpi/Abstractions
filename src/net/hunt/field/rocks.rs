//! The rocks the others broke: a piece taken here goes out, and a piece that comes back is quarried
//! in the same [`Taken`] and told to the world the same way, so the rock shrinks on both screens.

use abstractions_net::wire::Rock;
use bevy::prelude::*;

use crate::net::hunt::Hunted;
use crate::net::hunt::hooks::{self, Echoes, keep};
use crate::net::link::Link;
use crate::world::rocks::{RockId, RockTaken, Taken};

/// Returns `kilograms` as whole grams.
fn grams(kilograms: f32) -> u32 {
    (kilograms * 1000.0).round().clamp(0.0, u32::MAX as f32) as u32
}

/// Returns `grams` as kilograms.
fn kilograms(grams: u32) -> f32 {
    grams as f32 / 1000.0
}

/// Tells the server of each piece this machine took off a rock, but not of the ones it applied for
/// the others.
pub fn send(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    taken: Res<Taken>,
    mut told: MessageReader<RockTaken>,
) {
    for piece in told.read() {
        if let Some(at) = hunted.played.rocks.iter().position(|p| p == piece) {
            hunted.played.rocks.swap_remove(at);
            continue;
        }
        let whole = taken.whole(piece.id).unwrap_or(piece.kilograms);
        let rock = Rock {
            planet: piece.planet,
            id: piece.id.0,
            grams: grams(piece.kilograms),
            whole: grams(whole),
        };
        hooks::send_rock(&mut link, rock);
    }
}

/// Quarries `piece` of a rock of `whole` kilograms, and tells the world of it.
fn apply(
    (hunted, taken): (&mut Hunted, &mut Taken),
    told: &mut MessageWriter<RockTaken>,
    (piece, whole): (RockTaken, f32),
) {
    taken.quarry(piece.id, piece.kilograms, whole);
    keep(&mut hunted.played.rocks, piece);
    told.write(piece);
}

/// Takes in the pieces the others broke off rocks, and what was taken before this player joined.
pub fn play(
    (mut echoes, mut hunted): (ResMut<Echoes>, ResMut<Hunted>),
    mut taken: ResMut<Taken>,
    mut told: MessageWriter<RockTaken>,
) {
    for rock in std::mem::take(&mut echoes.rocks) {
        let piece = RockTaken {
            planet: rock.planet,
            id: RockId(rock.id),
            kilograms: kilograms(rock.grams),
        };
        apply(
            (&mut hunted, &mut taken),
            &mut told,
            (piece, kilograms(rock.whole)),
        );
    }
    for total in std::mem::take(&mut echoes.totals) {
        let id = RockId(total.id);
        let more = kilograms(total.grams) - taken.kilograms(id);
        let piece = RockTaken {
            planet: total.planet,
            id,
            kilograms: more,
        };
        if more > kilograms(1) {
            apply(
                (&mut hunted, &mut taken),
                &mut told,
                (piece, kilograms(total.whole)),
            );
        }
    }
}
