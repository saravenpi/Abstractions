//! Hunting in a session: this machine tells the server what its robot did to the animals, and the
//! blows, kills, carcasses, stuck spears and held items of the others come back into the world.
//!
//! The game keeps its own hunt (the blows of [`crate::avatar::strike::Hit`], the kill ledger and
//! the carcasses of [`crate::fauna::Herds`], the spears of [`crate::avatar::strike::Spear`]). The
//! systems of [`blows`], [`ledger`], [`spears`] and [`held`] read it and send what is new; the
//! systems of [`take`] write what arrives into the same places, so a remote kill is a kill like a
//! local one. [`Hunted`] remembers what was sent or taken, so nothing is sent back.

mod blows;
mod claim;
mod field;
mod held;
pub mod hooks;
mod item;
mod ledger;
mod spears;
mod take;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use abstractions_net::ids::BodyId;
use abstractions_net::log::PlayerId;
use abstractions_net::wire::{AnimalKey, Carcass, Event, Spear as Stuck, SpearKey};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub use hooks::Echoes;

use super::link::{FromServer, Link, Receive};
use crate::audio::Impact;
use crate::avatar::strike::{Hit, Spear};
use crate::fauna::senses::Focus;
use crate::fauna::{Beast, Broken, Herds, Lead};
use crate::space::Location;
use crate::world::Terrain;
use crate::world::rocks::RockTaken;

/// Seconds between two looks at the kill ledger and the carcasses for what is new.
const LOOK_EVERY: f32 = 0.25;

/// What this machine sent or took of the hunt, so that nothing it took goes back to the server.
#[derive(Resource, Debug, Default)]
pub struct Hunted {
    /// The carcasses sent or taken, by their key.
    known: HashSet<u64>,
    /// The parts taken of each carcass, as last sent or taken, by its key.
    taken: HashMap<u64, u64>,
    /// The parts broken of each animal, as last sent or taken.
    broken: HashMap<AnimalKey, u64>,
    /// Damage and broken parts that arrived while an animal was not drawn yet, by `(herd, slot)`.
    pub hurt: HashMap<(u64, u8), (f32, u64)>,
    /// The animals this machine struck lately, with the frames left to watch them break.
    victims: Vec<(Entity, u8)>,
    /// The spears that stand, with their key and the animal they are in.
    planted: HashMap<Entity, (SpearKey, Entity)>,
    /// The number the next spear of this machine gets.
    tag: u32,
    /// What this machine holds, by the id of its item in the pack, as last sent.
    held: Option<u32>,
    /// The wounds of animals plugged, as the animal and the part, sent or taken.
    plugged: HashSet<(AnimalKey, u8)>,
    /// What this machine played for the others, so that it is not sent back.
    played: Played,
    /// What the server told, not applied yet.
    inbox: Vec<Event>,
    /// Every carcass the server told of, on every body, until it is gone.
    lying: Vec<Carcass>,
    /// The body the carcasses were last applied on.
    at: Option<BodyId>,
    /// The spears the server told of that are not drawn yet.
    stuck: Vec<Stuck>,
    /// The spears pulled out by others, which are to leave the world.
    pulled: Vec<SpearKey>,
    /// The item each other player holds, as the bytes of its recipe.
    holding: HashMap<PlayerId, Vec<u8>>,
}

/// What the field acts of the others made this machine write into its own messages and components,
/// until the sender of the same message has seen it.
#[derive(Debug, Default)]
pub struct Played {
    /// The impacts sent to the audio.
    impacts: Vec<Impact>,
    /// The rock pieces told to the world.
    rocks: Vec<RockTaken>,
    /// The animals that began an attack for the others.
    attacks: Vec<Entity>,
}

/// Run condition: this machine struck an animal lately, so its parts may have broken.
fn watching(hunted: Res<Hunted>) -> bool {
    !hunted.victims.is_empty()
}

/// Run condition: the server told something that is not applied yet, a spear waits to be drawn, or
/// carcasses wait on a body the player just arrived at.
fn waiting(hunted: Res<Hunted>, location: Option<Res<Location>>) -> bool {
    !(hunted.inbox.is_empty() && hunted.stuck.is_empty() && hunted.pulled.is_empty())
        || location.is_some_and(|loc| hunted.at != Some(loc.body) && !hunted.lying.is_empty())
}

/// Registers the hunt of a session. Every system waits for a link, so single player runs none.
pub fn plugin(app: &mut App) {
    app.add_message::<Hit>()
        .init_resource::<Hunted>()
        .init_resource::<Echoes>()
        .add_systems(
            Update,
            reset.before(take::receive).run_if(resource_added::<Link>),
        )
        .add_plugins((takes, sends, field::plugin, claim::plugin));
}

/// Registers what is taken of the hunt of the others: the server's events, applied once the world
/// exists.
fn takes(app: &mut App) {
    let linked = resource_exists::<Link>;
    let look = || on_timer(Duration::from_secs_f32(LOOK_EVERY));
    let ready = resource_exists::<Terrain>
        .and_then(resource_exists::<Location>)
        .and_then(resource_exists::<Herds>);
    app.add_systems(
        Update,
        (
            take::receive
                .after(Receive)
                .run_if(linked)
                .run_if(on_message::<FromServer>),
            (take::apply.run_if(waiting), take::plant.run_if(waiting))
                .chain()
                .run_if(linked)
                .run_if(ready),
            born.run_if(linked).run_if(any_with_component::<Beast>),
            take::dress
                .run_if(linked)
                .run_if(look())
                .run_if(resource_exists::<Terrain>),
        )
            .chain(),
    );
}

/// Registers what this machine tells of its own hunt: blows, kills, carves, spears and what it holds.
fn sends(app: &mut App) {
    let linked = resource_exists::<Link>;
    let look = || on_timer(Duration::from_secs_f32(LOOK_EVERY));
    app.add_systems(
        Update,
        (
            blows::hits.run_if(linked).run_if(on_message::<Hit>),
            blows::breaks.run_if(linked).run_if(watching),
            (ledger::kills, ledger::carves)
                .run_if(linked)
                .run_if(resource_exists::<Herds>)
                .run_if(look()),
            spears::sticks
                .run_if(linked)
                .run_if(any_with_component::<Spear>),
            spears::pulls.run_if(linked),
            held::holds.run_if(linked),
        )
            .chain()
            .after(take::receive),
    );
}

/// Applies damage and broken parts that arrived before an animal was drawn.
fn born(
    mut hunted: ResMut<Hunted>,
    location: Option<Res<Location>>,
    mut beasts: Query<&mut Beast, Added<Beast>>,
) {
    let body = location.map(|loc| loc.body);
    for mut beast in &mut beasts {
        let Lead::Herd { id, slot } = beast.lead else {
            continue;
        };
        let prior = body
            .and_then(|body| {
                let key = AnimalKey {
                    body,
                    herd: id,
                    slot,
                };
                hunted.broken.get(&key).copied()
            })
            .unwrap_or(0);
        if let Some((joules, broken)) = hunted.hurt.remove(&(id, slot)) {
            let _ = beast.wits.health.hurt(joules);
            beast.wits.broken = Broken(beast.wits.broken.0 | broken | prior);
            beast.wits.provoked = beast.temper.grudge;
            if beast.wits.focus.is_none() {
                beast.wits.focus = Some(Focus {
                    at: Vec3::ZERO,
                    aware: 1.0,
                });
            }
        } else if prior != 0 {
            beast.wits.broken.0 |= prior;
        }
    }
}

/// Forgets what the last session sent or took.
fn reset(mut hunted: ResMut<Hunted>) {
    *hunted = Hunted::default();
}
