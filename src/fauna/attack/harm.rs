//! Harm: a blow that lands on an animal hurts it by its energy, pushes the part it struck, and breaks
//! that part when it carries more than the part can take. A blow that kills hands the animal to
//! [`super::slay`]: it falls and stays as a carcass.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::fracture::{breaks, detaches};
use super::gesture::Flinch;
use super::slay::slay;
use super::tune::FLINCH_PUSH;
use crate::avatar::strike::Hit;
use crate::craft::build::Intent;
use crate::craft::harvest::{Cut, Harvest};
use crate::craft::matter::Grade;
use crate::fauna::beast::Beast;
use crate::fauna::herds::Herds;
use crate::fauna::want::Lead;
use crate::life::Species;
use crate::space::{Anchor, Clock};
use crate::world::Terrain;

/// An animal that was killed: blows and spears pass through it while it lies.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Dead;

/// The blows an animal took, which grade its tissue when it dies.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Wounds {
    /// Blows landed.
    pub hits: u8,
    /// True when one of them crushed.
    pub blunt: bool,
}

/// What harm reads and writes besides the animals.
#[derive(SystemParam)]
pub struct Harm<'w> {
    herds: ResMut<'w, Herds>,
    clock: Res<'w, Clock>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
}

/// Counts the blow `hit` into the `wounds` of its victim and returns the count.
fn wound(commands: &mut Commands, hit: &Hit, wounds: Option<Mut<Wounds>>) -> Wounds {
    let blunt = hit.intent == Intent::Crush;
    match wounds {
        Some(mut w) => {
            w.hits = w.hits.saturating_add(1);
            w.blunt |= blunt;
            *w
        }
        None => {
            let first = Wounds { hits: 1, blunt };
            commands.entity(hit.victim).insert(first);
            first
        }
    }
}

/// Pushes the part `hit` struck of `beast` along the blow, by more the harder it hit.
fn flinch(
    commands: &mut Commands,
    (hit, anchor): (&Hit, &Anchor),
    (beast, species): (&Beast, &Species),
    part: usize,
) {
    let way = anchor.to_frame(hit.at + hit.along) - anchor.to_frame(hit.at);
    let along = beast.spot.rot.inverse() * way;
    let share = (hit.blow.energy / beast.wits.health.max.max(1e-3))
        .sqrt()
        .min(1.0);
    let push = along.as_vec3().normalize_or_zero() * FLINCH_PUSH * species.body.length * share;
    commands.entity(hit.victim).insert(Flinch {
        part: part as u8,
        push,
        age: 0.0,
    });
}

/// Breaks the part `index` of `beast` when the blow carried more than it can take: it is marked
/// broken, and a part on a stalk of its own falls off and gives its tissue at once.
fn snap(
    commands: &mut Commands,
    (hit, wounds): (&Hit, Wounds),
    (beast, species): (&mut Beast, &Species),
    (index, asks): (usize, &mut MessageWriter<Harvest>),
) {
    if beast.wits.broken.has(index) || !breaks(species, index, hit.blow.energy) {
        return;
    }
    beast.wits.broken = beast.wits.broken.with(index);
    if detaches(species.body.parts[index].kind) {
        commands.entity(hit.part).despawn();
        beast.parts[index] = None;
        asks.write(Harvest(Cut {
            species: beast.index,
            part: index as u8,
            kill: Grade::of(wounds.hits, wounds.blunt),
        }));
    }
}

/// Takes the energy of each blow that landed from the health of the animal struck, pushes and maybe
/// breaks the part it struck, and lets an animal with no health left fall and become a carcass.
pub fn harm(
    mut commands: Commands,
    mut hits: MessageReader<Hit>,
    mut ctx: Harm,
    mut beasts: Query<(&mut Beast, Option<&mut Wounds>), Without<Dead>>,
    mut asks: MessageWriter<Harvest>,
) {
    let globe = ctx.terrain.globe().clone();
    let Some(life) = globe.life() else { return };
    for hit in hits.read() {
        let Ok((mut beast, wounds)) = beasts.get_mut(hit.victim) else {
            continue;
        };
        let species = life.species.get(usize::from(beast.index));
        if !matches!(beast.lead, Lead::Herd { .. }) {
            continue;
        }
        let now = wound(&mut commands, hit, wounds);
        let part = beast.parts.iter().position(|p| *p == Some(hit.part));
        if let (Some(species), Some(part)) = (species, part) {
            flinch(&mut commands, (hit, &*ctx.anchor), (&*beast, species), part);
            let broke = (&mut *beast, species);
            snap(&mut commands, (hit, now), broke, (part, &mut asks));
        }
        if beast.wits.health.hurt(hit.blow.energy) {
            let (herds, kill) = (&mut *ctx.herds, Grade::of(now.hits, now.blunt));
            let dead = (hit.victim, &mut *beast);
            slay(
                &mut commands,
                dead,
                (herds, ctx.clock.seconds),
                (&*ctx.terrain, kill),
            );
        }
    }
}
