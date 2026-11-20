//! The attacks of animals, for everyone: the client of the player an animal goes for says which
//! attack starts, and the others start the same one, so the wind-up and the lunge play in step.

use abstractions_net::wire::Attack;
use bevy::prelude::*;

use super::tune::ATTACK_LATE;
use crate::fauna::{Attacking, Beast, Creature, Dead, Kind, Lead};
use crate::net::hunt::Hunted;
use crate::net::hunt::blows::key_of;
use crate::net::hunt::hooks::{self, Echoes};
use crate::net::link::Link;
use crate::space::{Clock, Location};
use crate::world::Terrain;

/// Returns the number of an attack on the network: its position in [`Kind::ORDER`].
fn code(kind: Kind) -> u8 {
    Kind::ORDER.iter().position(|k| *k == kind).unwrap_or(0) as u8
}

/// Run condition: some animal began an attack this frame.
pub fn began(started: Query<(), Added<Attacking>>) -> bool {
    !started.is_empty()
}

/// Tells the server of each attack an animal began against this robot, but not of the ones it
/// started for the others.
pub fn send(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    (location, clock): (Res<Location>, Res<Clock>),
    started: Query<(Entity, &Creature, &Attacking), Added<Attacking>>,
) {
    let Some(you) = link.you else {
        return;
    };
    for (entity, creature, attacking) in &started {
        if let Some(at) = hunted.played.attacks.iter().position(|e| *e == entity) {
            hunted.played.attacks.swap_remove(at);
            continue;
        }
        let attack = Attack {
            animal: key_of(&location, creature),
            kind: code(attacking.kind),
            target: Some(you),
            started: clock.seconds,
        };
        hooks::send_attack(&mut link, attack);
    }
}

/// Returns the attack `told` makes an animal of `beast` begin on this machine, `now` seconds into the
/// universe, or nothing when the attack is unknown or too late.
fn begin(told: &Attack, (beast, terrain): (&Beast, &Terrain), now: f64) -> Option<Attacking> {
    let globe = terrain.globe();
    let species = globe.life()?.species.get(usize::from(beast.index))?;
    let late = now - told.started;
    let kind = Kind::ORDER
        .get(usize::from(told.kind))
        .copied()
        .filter(|_| late < ATTACK_LATE)?;
    let mut attack = Attacking::begin(kind, species.body.length, globe.gravity() as f32);
    attack.t = late.max(0.0) as f32;
    Some(attack)
}

/// The animals drawn here that are alive and not attacking.
type Idle<'w, 's> = Query<'w, 's, (Entity, &'static Beast), (Without<Attacking>, Without<Dead>)>;

/// Starts the attacks the others told on the animals drawn here, which are not already attacking.
pub fn play(
    (mut echoes, mut hunted): (ResMut<Echoes>, ResMut<Hunted>),
    (location, clock, terrain): (Res<Location>, Res<Clock>, Res<Terrain>),
    beasts: Idle,
    mut commands: Commands,
) {
    for told in std::mem::take(&mut echoes.attacks) {
        let (herd, slot) = (told.animal.herd, told.animal.slot);
        let drawn = |(_, b): &(Entity, &Beast)| matches!(b.lead, Lead::Herd { id, slot: s } if id == herd && s == slot);
        let found = beasts.iter().find(drawn);
        let begun = found.and_then(|(e, b)| Some((e, begin(&told, (b, &terrain), clock.seconds)?)));
        let Some((entity, attack)) = begun.filter(|_| told.animal.body == location.body) else {
            continue;
        };
        commands.entity(entity).insert(attack);
        hooks::keep(&mut hunted.played.attacks, entity);
    }
}
