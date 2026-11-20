//! Carving: the interact key cuts the part of a carcass the robot looks at. The tool rule of the
//! harvest decides what it gives, and the part is gone from the body for everyone who sees it.

use bevy::camera::primitives::Aabb;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Dead;
use super::slay::Carved;
use super::tune::{CUT_RANGE, CUT_SLACK};
use crate::craft::harvest::{Cut, Harvest};
use crate::craft::matter::{Class, Grade};
use crate::fauna::beast::{Beast, Part};
use crate::fauna::herds::Herds;
use crate::fauna::want::Lead;
use crate::life::Species;
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};
use crate::world::Terrain;

/// What reads the key and the view.
#[derive(SystemParam)]
pub struct Eyes<'w, 's> {
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    settings: Res<'w, Persistent<Settings>>,
}

/// The parts of the animals as the cut sees them.
type Parts<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        &'static Aabb,
        &'static ChildOf,
    ),
    With<Part>,
>;

/// Returns the part of a body among `parts` that the line from `from` along `ahead` passes closest
/// to, within [`CUT_RANGE`], when `dead` says its animal is dead: the part and its animal.
fn aimed(
    parts: &Parts,
    dead: &Query<&mut Beast, With<Dead>>,
    (from, ahead): (Vec3, Vec3),
) -> Option<(Entity, Entity)> {
    let near = parts.iter().filter_map(|(part, placed, aabb, of)| {
        dead.contains(of.parent()).then_some(())?;
        let rel = placed.transform_point(Vec3::from(aabb.center)) - from;
        let along = rel.dot(ahead);
        let off = (rel - ahead * along).length();
        let size = Vec3::from(aabb.half_extents).length() * placed.scale().max_element();
        let seen = (0.0..=CUT_RANGE).contains(&along) && off <= size + CUT_SLACK;
        seen.then_some((off, part, of.parent()))
    });
    near.min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, p, b)| (p, b))
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

/// Returns true when part `index` of `species` has tissue worth taking.
fn worth(species: &Species, index: usize) -> bool {
    let part = species.body.parts.get(index);
    let tissue = part.and_then(|p| species.tissues.get(usize::from(p.tissue)).map(|t| (t, p)));
    tissue.is_some_and(|(t, p)| Class::of(t.kind, p.kind).is_some())
}

/// Cuts the part of the carcass the robot looks at when the interact key goes down.
pub fn carve(
    eyes: Eyes,
    (mut herds, terrain): (ResMut<Herds>, Res<Terrain>),
    (mut commands, mut asks): (Commands, MessageWriter<Harvest>),
    (mut dead, parts): (Query<&mut Beast, With<Dead>>, Parts),
) {
    let key = eyes.settings.controls.keys.ship.board;
    let (Ok(eye), true) = (eyes.camera.single(), eyes.keys.just_pressed(key)) else {
        return;
    };
    let ahead = eye.forward().as_vec3();
    let Some((part, body)) = aimed(&parts, &dead, (eye.translation(), ahead)) else {
        return;
    };
    let Ok(mut beast) = dead.get_mut(body) else {
        return;
    };
    let globe = terrain.globe().clone();
    let species = globe
        .life()
        .and_then(|l| l.species.get(usize::from(beast.index)));
    let index = beast.parts.iter().position(|p| *p == Some(part));
    if let (Some(species), Some(index)) = (species, index)
        && worth(species, index)
    {
        let cut = Cut {
            species: beast.index,
            part: index as u8,
            kill: kill(&herds, beast.lead),
        };
        asks.write(Harvest(cut));
        note(&mut herds, beast.lead, index);
        commands.entity(part).despawn();
        beast.parts[index] = None;
    }
}
