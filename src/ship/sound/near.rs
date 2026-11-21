//! Which ships of other players are heard, when a ship faster than sound booms, and where the
//! robots can stand on a ship and hear their feet on its wing.

use bevy::prelude::*;

use super::super::exhaust::Exhaust;
use super::super::mesh::hulls;
use crate::audio::{Ch, HULLS, HullSound};
use crate::avatar::tread::Hulls;
use crate::space::Placed;

/// Distance in metres at which the boom of a ship is at its full strength.
const BOOM_REACH: f32 = 150.0;

/// How many times its own speed a ship may seem to close in before the reading is taken for
/// a change of ship in the slot, not for a flight.
const SLACK: f32 = 1.2;

/// Puts `hull` among `slots`, which stay sorted from the nearest ship to the farthest. A ship
/// that is not there, or farther than every ship of full slots, is left out.
pub fn admit(slots: &mut [HullSound], mut hull: HullSound) {
    for slot in slots {
        if hull.at(Ch::Present) < 0.5 {
            return;
        }
        let empty = slot.at(Ch::Present) < 0.5;
        if empty || hull.at(Ch::Distance) < slot.at(Ch::Distance) {
            std::mem::swap(slot, &mut hull);
        }
    }
}

/// Returns how fast `hull` closes in on the listener as a Mach number, from its distance
/// `before`, `dt` seconds ago. Zero for a ship that is not there, has no air to carry a shock,
/// or was not in this slot before.
pub fn closing(before: f32, hull: &HullSound, dt: f32) -> f32 {
    let pace = (before - hull.at(Ch::Distance)) / dt.max(1.0e-4);
    let there = hull.at(Ch::Present) > 0.5 && hull.at(Ch::Air) > 0.0;
    if !there || pace > SLACK * hull.at(Ch::Speed) {
        return 0.0;
    }
    pace / hull.at(Ch::Sound)
}

/// Returns the strength of the boom of every ship whose shock wave passes the listener now,
/// and keeps the distances in `last` for the next frame.
///
/// A ship that closes in faster than sound outruns its own sound. The moment its closing speed
/// falls back through the speed of sound is the moment its Mach cone sweeps over the listener.
pub fn booms<'a>(
    hulls: &'a [HullSound; HULLS],
    last: &'a mut [(f32, f32); HULLS],
    dt: f32,
) -> impl Iterator<Item = f32> + 'a {
    hulls
        .iter()
        .zip(last.iter_mut())
        .filter_map(move |(hull, last)| {
            let now = closing(last.0, hull, dt);
            let passed = last.1 >= 1.0 && now < 1.0;
            let outside = hull.at(Ch::Present) > 0.5 && hull.at(Ch::Inside) < 0.5;
            *last = (hull.at(Ch::Distance), now);
            (passed && outside).then(|| (BOOM_REACH / hull.at(Ch::Distance).max(1.0)).min(1.0))
        })
}

/// Writes where every ship is for the feet of the robots: the centre of its hull, and the reach
/// of its airframe, the farthest corner of its parts from that centre, worked out once.
pub fn decks(
    ships: Query<&Placed, With<Exhaust>>,
    mut decks: ResMut<Hulls>,
    mut reach: Local<f32>,
) {
    if *reach <= 0.0 {
        *reach = hulls()
            .iter()
            .flatten()
            .map(|corner| corner.length())
            .fold(0.0, f32::max);
    }
    decks.0.clear();
    decks
        .0
        .extend(ships.iter().map(|placed| (placed.pos, *reach)));
}
