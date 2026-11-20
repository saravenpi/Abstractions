//! The hooks of the systems that other branches add: each is one call that sends a record to the
//! server, and what comes back waits in [`Echoes`] for the system that plays it. Nothing here reads
//! a game type, so a branch connects its own with one line.
//!
//! - [`send_blow`] and [`Echoes::blows`]: a blow on a robot. The sender is [`super::field`]'s duel,
//!   where a swing enters the puppet of another player; the victim writes the blow it reads from
//!   the echoes as an `avatar::blow::Blow`, which harms, shoves and staggers it.
//! - [`send_wound`], [`send_plug`], [`Echoes::wounds`], [`Echoes::plugs`]: wounds of animals, from
//!   `fauna::bleed`.
//! - [`send_impact`] and [`Echoes::impacts`]: the `Impact` message, for sound and chips.
//! - [`send_attack`] and [`Echoes::attacks`]: the attack an animal starts. Animals are a function of
//!   the seed and the shared clock, and the client of the player an animal goes for says which
//!   attack starts, so everyone plays the same wind-up.
//! - [`send_rock`], [`Echoes::rocks`], [`Echoes::totals`]: the `RockTaken` message. The server adds
//!   the pieces up and sends the total of each rock on joining, so a taken rock stays gone.

use abstractions_net::log::PlayerId;
use abstractions_net::wire::{Act, AnimalKey, Attack, Blow, ClientMsg, Impact, Rock, Wound};
use bevy::prelude::*;

use crate::net::link::Link;

/// Most echoes of one kind kept. The oldest go first, so a game that never reads them does not
/// grow.
pub const ECHOES_MAX: usize = 1024;

/// What the server told that a system of another branch plays. Each list is read and cleared by it.
#[derive(Resource, Debug, Default)]
pub struct Echoes {
    /// Blows on the robot of this player, with the player who struck.
    pub blows: Vec<(PlayerId, Blow)>,
    /// Wounds opened on animals by the others, and those already open when this player joined.
    pub wounds: Vec<(AnimalKey, Wound)>,
    /// Wounds the others plugged: the animal and the part.
    pub plugs: Vec<(AnimalKey, u8)>,
    /// Swings and throws of the others that landed.
    pub impacts: Vec<Impact>,
    /// Attacks animals started.
    pub attacks: Vec<Attack>,
    /// Pieces the others broke off rocks.
    pub rocks: Vec<Rock>,
    /// What was taken of each rock before this player joined, as one record of the total each.
    pub totals: Vec<Rock>,
}

/// Keeps `item` at the end of `list`, dropping the oldest when [`ECHOES_MAX`] are kept.
pub fn keep<T>(list: &mut Vec<T>, item: T) {
    if list.len() >= ECHOES_MAX {
        list.remove(0);
    }
    list.push(item);
}

/// Tells the server a blow landed on the robot of another player.
pub fn send_blow(link: &mut Link, blow: Blow) {
    link.send(ClientMsg::Hunt(Act::Blow(blow)));
}

/// Tells the server a wound opened on a part of an animal.
pub fn send_wound(link: &mut Link, animal: AnimalKey, wound: Wound) {
    link.send(ClientMsg::Hunt(Act::Wound { animal, wound }));
}

/// Tells the server a spear plugged the wound on `part` of an animal.
pub fn send_plug(link: &mut Link, animal: AnimalKey, part: u8) {
    link.send(ClientMsg::Hunt(Act::Plug { animal, part }));
}

/// Tells the server where a swing or a throw landed.
pub fn send_impact(link: &mut Link, impact: Impact) {
    link.send(ClientMsg::Hunt(Act::Impact(impact)));
}

/// Tells the server an animal starts an attack.
pub fn send_attack(link: &mut Link, attack: Attack) {
    link.send(ClientMsg::Hunt(Act::Attack(attack)));
}

/// Tells the server a rock was taken.
pub fn send_rock(link: &mut Link, rock: Rock) {
    link.send(ClientMsg::Hunt(Act::Rock(rock)));
}
