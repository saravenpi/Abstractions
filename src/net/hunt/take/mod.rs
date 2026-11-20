//! Applying what the server tells of the hunt of the others: the blows, breaks, kills and cuts
//! happen to the animals of this machine too, the spears they planted stand in them, and what
//! they hold is in the hands of their puppets.
//!
//! [`receive`] only keeps what arrives; [`apply`] runs a few times a second, once the world
//! exists, because the welcome of a server comes before the world is built.

mod animals;
mod held;
mod pick;
mod spears;

pub use held::dress;
pub use spears::plant;

use abstractions_net::wire::{Event as Told, HuntWorld, ServerMsg};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::hooks::keep;
use super::{Echoes, Hunted};
use crate::fauna::{Beast, Herds};
use crate::net::link::FromServer;
use crate::space::{Clock, Location};
use crate::world::Terrain;

/// Most events kept while the world is not ready. The oldest go first.
const INBOX_MAX: usize = 4096;

/// What applying an event to the animals reads and writes.
#[derive(SystemParam)]
pub struct Land<'w, 's> {
    /// Spawns and despawns.
    commands: Commands<'w, 's>,
    /// What was sent or taken.
    hunted: ResMut<'w, Hunted>,
    /// The kill ledger and the carcasses.
    herds: ResMut<'w, Herds>,
    /// The planet underfoot.
    terrain: Res<'w, Terrain>,
    /// The animals that are drawn.
    beasts: Query<'w, 's, (Entity, &'static mut Beast)>,
}

/// Keeps the hunting events of the server that arrived this frame. What other branches play goes
/// to the echoes at once, since it needs no world; the rest waits for [`apply`].
pub fn receive(
    mut from: MessageReader<FromServer>,
    (mut hunted, mut echoes): (ResMut<Hunted>, ResMut<Echoes>),
) {
    for FromServer(msg) in from.read() {
        let ServerMsg::Hunt(event) = msg else {
            continue;
        };
        match event.clone() {
            Told::Blow { by, blow } => keep(&mut echoes.blows, (by, blow)),
            Told::Wound { animal, wound } => keep(&mut echoes.wounds, (animal, wound)),
            Told::Plug { animal, part } => keep(&mut echoes.plugs, (animal, part)),
            Told::Impact(impact) => keep(&mut echoes.impacts, impact),
            Told::Attack(attack) => keep(&mut echoes.attacks, attack),
            Told::Rock(rock) => keep(&mut echoes.rocks, rock),
            Told::World(mut world) => {
                echoes.wounds.extend(std::mem::take(&mut world.wounds));
                echoes.totals.extend(std::mem::take(&mut world.rocks));
                hunted.inbox.push(Told::World(world));
            }
            other => hunted.inbox.push(other),
        }
        if hunted.inbox.len() > INBOX_MAX {
            hunted.inbox.remove(0);
        }
    }
}

/// Applies what the server told, to the animals of the body the player is at. A carcass of
/// another body waits in the list of carcasses until the player goes there.
pub fn apply(mut land: Land, (location, clock): (Res<Location>, Res<Clock>)) {
    let now = clock.seconds;
    land.hunted
        .lying
        .retain(|c| now < c.since + f64::from(c.lasts));
    if land.hunted.at != Some(location.body) {
        land.hunted.at = Some(location.body);
        land.hunted.known.clear();
        land.hunted.taken.clear();
        let lying = land.hunted.lying.clone();
        for carcass in lying.iter().filter(|c| c.animal.body == location.body) {
            animals::lay(&mut land, carcass);
        }
    }
    for event in std::mem::take(&mut land.hunted.inbox) {
        route(&mut land, &location, event);
    }
}

/// Applies one event.
fn route(land: &mut Land, location: &Location, event: Told) {
    match event {
        Told::World(world) => lay_world(land, location, world),
        Told::Hit { hit, .. } => animals::hurt(land, location, &hit),
        Told::Broke { animal, part } => animals::broke(land, location, (animal, part)),
        Told::Killed(carcass) => animals::killed(land, location, carcass),
        Told::Carved { animal, part, .. } => animals::carved(land, location, (animal, part)),
        Told::Stuck(spear) => land.hunted.stuck.push(spear),
        Told::Pulled { key, .. } => land.hunted.pulled.push(key),
        Told::Held {
            by,
            item: Some(item),
        } => {
            land.hunted.holding.insert(by, item);
        }
        Told::Held { by, item: None } => {
            land.hunted.holding.remove(&by);
        }
        Told::Blow { .. }
        | Told::Wound { .. }
        | Told::Plug { .. }
        | Told::Impact(_)
        | Told::Attack(_)
        | Told::Rock(_) => {}
    }
}

/// Applies the state of the hunt a server tells after joining.
fn lay_world(land: &mut Land, location: &Location, world: HuntWorld) {
    land.hunted.stuck.extend(world.spears);
    for (by, item) in world.held {
        land.hunted.holding.insert(by, item);
    }
    for carcass in world.carcasses {
        animals::killed(land, location, carcass);
    }
}
