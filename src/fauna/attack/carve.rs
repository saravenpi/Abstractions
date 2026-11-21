//! Carving: the interact key cuts the part of a carcass the robot looks at within its reach, with a
//! tool or bare hands, when the prompt names that cut and what it gives. The tool rule of the harvest decides what it gives, and the part is gone from the body
//! for everyone who sees it.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::aim::{Carcasses, Parts, Target, named, target};
use super::slay::Carved;
use super::tune::CARVE;
use crate::audio::{Body, Impact};
use crate::craft::Held;
use crate::craft::harvest::{Claims, Cut, Harvest, harvest, tool};
use crate::craft::matter::Grade;
use crate::fauna::beast::Beast;
use crate::fauna::bleed::impact;
use crate::fauna::herds::Herds;
use crate::fauna::want::Lead;
use crate::life::{Life, Species};
use crate::player::{Interact, Player, Verb};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};
use crate::sim::PlayerId;
use crate::world::Terrain;

/// What reads the key and the view: the camera, the local robot whose reach the cut is, the keys,
/// their bindings, the action the interact key does and the tool in the hand.
#[derive(SystemParam)]
pub struct Eyes<'w, 's> {
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    robots: Query<'w, 's, (&'static GlobalTransform, &'static PlayerId), With<Player>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    settings: Res<'w, Persistent<Settings>>,
    interact: ResMut<'w, Interact>,
    held: Option<Res<'w, Held>>,
}

impl Eyes<'_, '_> {
    /// Returns where the local robot stands, when it does.
    fn robot(&self) -> Option<Vec3> {
        let mine = self.robots.iter().find(|(_, id)| **id == PlayerId::LOCAL);
        mine.map(|(placed, _)| placed.translation())
    }

    /// Offers `cut` of a carcass of `species` of `life` on the planet `seed`, named with what it
    /// gives with the tool in the hand, bare hands giving less, and returns true when the interact
    /// key takes it now.
    fn offer(&mut self, (life, seed): (&Life, u64), (species, cut): (&Species, Cut)) -> bool {
        let tool = self.held.as_deref().and_then(tool);
        let grams = harvest(life, seed, cut, tool).map(|stack| (stack.mass * 1e3).round() as u32);
        let name = named(species, usize::from(cut.part));
        self.interact.weighed(Verb::Carve, name, grams);
        let key = self.settings.controls.keys.ship.board;
        self.interact.pressed(Verb::Carve, &self.keys, key)
    }
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

/// Returns the cut to harvest now: at once alone, or nothing in a session, where it waits in
/// `claims` for the server to give the part `index` of the carcass `lead` to this player.
fn claim(claims: Option<&mut Claims>, lead: Lead, (index, cut): (usize, Cut)) -> Option<Cut> {
    let (Some(claims), Lead::Carcass { key, .. }) = (claims, lead) else {
        return Some(cut);
    };
    claims.claim((key, index as u8), cut)
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
/// down while the prompt names that cut. The part leaves the body at once; in a session its tissue
/// enters the pack once the server gives it to this player.
pub fn carve(
    mut eyes: Eyes,
    (mut herds, terrain, mut claims): (ResMut<Herds>, Res<Terrain>, Option<ResMut<Claims>>),
    (mut commands, mut asks, mut sounds): (Commands, MessageWriter<Harvest>, MessageWriter<Impact>),
    (mut dead, parts): (Carcasses, Parts),
) {
    let globe = terrain.globe().clone();
    let (Ok(eye), Some(life)) = (eyes.camera.single().copied(), globe.life()) else {
        return;
    };
    let Some(Target { part, body, index }) = target((&eye, eyes.robot()), (&parts, &dead), life)
    else {
        return;
    };
    let Ok(mut beast) = dead.get_mut(body) else {
        return;
    };
    let Some(species) = life.species.get(usize::from(beast.index)) else {
        return;
    };
    let cut = cut_of(&herds, &beast, index);
    if !eyes.offer((life, terrain.seed()), (species, cut)) {
        return;
    }
    let at = parts
        .get(part)
        .map_or(eye.translation(), |p| p.1.translation());
    sounds.write(knife(species, index, at));
    if let Some(cut) = claim(claims.as_deref_mut(), beast.lead, (index, cut)) {
        asks.write(Harvest(cut));
    }
    note(&mut herds, beast.lead, index);
    commands.entity(part).despawn();
    beast.parts[index] = None;
}
