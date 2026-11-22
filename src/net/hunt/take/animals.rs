//! What the others did to animals, done to the animals of this machine: a blow hurts, a part
//! breaks, a kill lays the carcass and a cut takes the part off it.

use abstractions_net::wire::{AnimalKey, Carcass as Lying, Hit};
use bevy::math::{DVec3, Vec3};

use super::Land;
use super::pick::{bit, find};
use crate::craft::matter::Grade;
use crate::fauna::senses::Focus;
use crate::fauna::{Carved, carcass_key, slay};
use crate::life::hunt;
use crate::space::Location;

/// Hurts the animal `hit` struck, when it is drawn here, or remembers the damage until it spawns:
/// it limps when wounded and goes for its attacker. Whether it dies is told by the kill.
pub fn hurt(land: &mut Land, location: &Location, hit: &Hit) {
    if hit.animal.body != location.body {
        return;
    }
    if let Some((_, mut beast)) = find(&mut land.beasts, &hit.animal, None) {
        let _ = beast.wits.health.hurt(hit.joules);
        beast.wits.provoked = beast.temper.grudge;
        if beast.wits.focus.is_none() {
            beast.wits.focus = Some(Focus {
                at: Vec3::ZERO,
                aware: 1.0,
            });
        }
    } else {
        let entry = land
            .hunted
            .hurt
            .entry((hit.animal.herd, hit.animal.slot))
            .or_default();
        entry.0 += hit.joules;
    }
}

/// Breaks `part` of `animal`, when it is on this body.
pub fn broke(land: &mut Land, location: &Location, (animal, part): (AnimalKey, u8)) {
    if animal.body != location.body {
        return;
    }
    *land.hunted.broken.entry(animal).or_insert(0) |= bit(part);
    if let Some((_, mut beast)) = find(&mut land.beasts, &animal, None) {
        beast.wits.broken.0 |= bit(part);
    } else {
        land.hunted
            .hurt
            .entry((animal.herd, animal.slot))
            .or_default()
            .1 |= bit(part);
    }
}

/// Keeps the carcass `carcass` in the list of carcasses and, when it lies on this body, lays it.
pub fn killed(land: &mut Land, location: &Location, carcass: Lying) {
    let others = |c: &Lying| c.animal != carcass.animal;
    land.hunted.lying.retain(others);
    land.hunted.lying.push(carcass.clone());
    if carcass.animal.body == location.body {
        lay(land, &carcass);
    }
}

/// Returns the carcass the ledger of this machine would hold for `lying`, with its key.
fn made(land: &Land, lying: &Lying) -> Option<(hunt::Carcass, u64)> {
    let life = land.terrain.globe().life()?;
    let animal = lying.animal;
    let at = DVec3::from_array(lying.at);
    let made = hunt::slain(
        life,
        (animal.herd, animal.slot, lying.species),
        at,
        lying.since,
    );
    let key = carcass_key(&made);
    Some((made, key))
}

/// Lays the carcass `lying` here: the animal falls where it stands, or the carcass is added to the
/// ledger when the animal is not drawn, then the parts broken and taken are applied. A carcass
/// that is already here, as the killer's own is, is left as it is.
pub fn lay(land: &mut Land, lying: &Lying) {
    let Some((made, key)) = made(land, lying) else {
        return;
    };
    let here = land.herds.slain.iter().any(|c| carcass_key(c) == key);
    if !land.hunted.known.insert(key) || here {
        return;
    }
    let animal = lying.animal;
    bury(land, lying, (made, key));
    if let Some((_, mut beast)) = find(&mut land.beasts, &animal, Some(key)) {
        beast.wits.broken.0 |= lying.broken;
    }
    cut(land, &animal, (key, lying.taken));
}

/// Lays the animal of `lying` dead: it falls where it stands when it is drawn, else the ledger
/// takes the carcass `made` of identity `key`.
fn bury(land: &mut Land, lying: &Lying, (made, key): (hunt::Carcass, u64)) {
    let grade = match lying.grade {
        0 => Grade::Clean,
        1 => Grade::Good,
        _ => Grade::Poor,
    };
    let animal = lying.animal;
    match find(&mut land.beasts, &animal, None) {
        Some((entity, mut beast)) => {
            let (herds, terrain) = (&mut *land.herds, &*land.terrain);
            let dead = (entity, &mut *beast);
            slay(
                &mut land.commands,
                dead,
                (herds, lying.since),
                (terrain, grade),
            );
        }
        None => {
            land.herds
                .ledger
                .kill(animal.herd, animal.slot, lying.since);
            let carved = Carved {
                key,
                taken: 0,
                kill: grade,
            };
            land.herds.carved.push(carved);
            land.herds.slain.push(made);
        }
    }
}

/// Takes the parts of `mask` off the carcass `key` of `animal`, in the ledger and from its body.
fn cut(land: &mut Land, animal: &AnimalKey, (key, mask): (u64, u64)) {
    let seen = land.hunted.taken.entry(key).or_insert(0);
    let fresh = mask & !*seen;
    *seen |= fresh;
    let held = land.herds.carved.iter_mut().filter(|c| c.key == key);
    held.for_each(|c| c.taken |= fresh);
    let Some((_, mut beast)) = find(&mut land.beasts, animal, Some(key)) else {
        return;
    };
    for part in (0..64usize).filter(|part| fresh >> part & 1 == 1) {
        if let Some(entity) = beast.parts.get_mut(part).and_then(Option::take) {
            land.commands.entity(entity).despawn();
        }
    }
}

/// Takes `part` off the carcass of `animal`, when it lies on this body.
pub fn carved(land: &mut Land, location: &Location, (animal, part): (AnimalKey, u8)) {
    if animal.body != location.body {
        return;
    }
    let Some(carcass) = land.hunted.lying.iter_mut().find(|c| c.animal == animal) else {
        return;
    };
    carcass.taken |= bit(part);
    let lying = carcass.clone();
    if let Some((_, key)) = made(land, &lying) {
        cut(land, &animal, (key, bit(part)));
    }
}
