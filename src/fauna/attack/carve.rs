//! Carving: the interact key cuts the part of a carcass the robot looks at, when the prompt names
//! that cut. The tool rule of the harvest decides what it gives, and the part is gone from the body
//! for everyone who sees it.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::aim::{Carcasses, Parts, Target, named, target};
use super::slay::Carved;
use super::tune::CARVE;
use crate::audio::{Body, Impact};
use crate::craft::harvest::{Cut, Harvest};
use crate::craft::matter::Grade;
use crate::fauna::beast::Beast;
use crate::fauna::bleed::impact;
use crate::fauna::herds::Herds;
use crate::fauna::want::Lead;
use crate::life::Species;
use crate::player::{Interact, Verb};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};
use crate::world::Terrain;

/// What reads the key and the view.
#[derive(SystemParam)]
pub struct Eyes<'w, 's> {
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    settings: Res<'w, Persistent<Settings>>,
    interact: ResMut<'w, Interact>,
}

/// Returns the grade of the kill of the carcass `lead` of `herds`, poor when it is not recorded.
fn kill(herds: &Herds, lead: Lead) -> Grade {
    let Lead::Carcass { key, .. } = lead else {
        return Grade::Poor;
    };
    herds
        .carved
        .iter()
        .find(|c| c.key == key)
        .map_or(Grade::Poor, |c| c.kill)
}

/// Records that part `index` of the carcass `lead` was taken.
fn note(herds: &mut Herds, lead: Lead, index: usize) {
    let Lead::Carcass { key, .. } = lead else {
        return;
    };
    let mark = |c: &mut Carved| c.taken |= 1 << index.min(63);
    herds
        .carved
        .iter_mut()
        .filter(|c| c.key == key)
        .for_each(mark);
}

/// Returns the cut of part `index` of the carcass `beast`, graded by how it was killed.
fn cut_of(herds: &Herds, beast: &Beast, index: usize) -> Cut {
    Cut {
        species: beast.index,
        part: index as u8,
        kill: kill(herds, beast.lead),
    }
}

/// Returns the sound of the flint knife cutting part `index` of the carcass of `species` at `at`.
fn knife(species: &Species, index: usize, at: Vec3) -> Impact {
    let edge = Body {
        stiffness: CARVE.1,
        mass: CARVE.2,
    };
    impact(species, index, (at, CARVE.0, true), edge)
}

/// Offers to cut the part of the carcass the robot looks at, and cuts it when the interact key goes
/// down while the prompt names that cut.
pub fn carve(
    mut eyes: Eyes,
    (mut herds, terrain): (ResMut<Herds>, Res<Terrain>),
    (mut commands, mut asks, mut sounds): (Commands, MessageWriter<Harvest>, MessageWriter<Impact>),
    (mut dead, parts): (Carcasses, Parts),
) {
    let globe = terrain.globe().clone();
    let (Ok(eye), Some(life)) = (eyes.camera.single().copied(), globe.life()) else {
        return;
    };
    let Some(Target { part, body, index }) = target(&eye, (&parts, &dead), life) else {
        return;
    };
    let Ok(mut beast) = dead.get_mut(body) else {
        return;
    };
    let Some(species) = life.species.get(usize::from(beast.index)) else {
        return;
    };
    eyes.interact.offer(Verb::Carve, named(species, index));
    let key = eyes.settings.controls.keys.ship.board;
    if !eyes.interact.pressed(Verb::Carve, &eyes.keys, key) {
        return;
    }
    let at = parts
        .get(part)
        .map_or(eye.translation(), |p| p.1.translation());
    sounds.write(knife(species, index, at));
    asks.write(Harvest(cut_of(&herds, &beast, index)));
    note(&mut herds, beast.lead, index);
    commands.entity(part).despawn();
    beast.parts[index] = None;
}
