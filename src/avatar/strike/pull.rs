//! Taking a thrown tool again: only the interact key does it, through the one prompt. A javelin
//! that lies on the ground is taken, one planted in the ground pulled out, when the robot looks at
//! it within reach; one stuck in an animal or a carcass is pulled out when the robot stands by it.
//! Walking over a javelin leaves it where it is. One of the robot's own pack goes back to its
//! hotbar slot whatever the hands hold; one of another player needs empty hands.

use bevy::prelude::*;

use super::flight::{Rest, Spear};
use super::held::Held;
use super::input::Robot;
use super::javelin::tune::{TAKE_AIM, TAKE_REACH};
use super::tune::PULL;
use crate::craft::Pack;
use crate::fauna::Creature;
use crate::player::{Interact, Player, Verb};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};

/// What the robot's reach reads: the players, the hands that are full and the animals.
pub type Reach<'w, 's> = (
    Query<'w, 's, &'static Transform, With<Player>>,
    Query<'w, 's, (), With<Held>>,
    Query<'w, 's, (), With<Creature>>,
);

/// What a take reads and writes: the keys, the settings that bind them, the pack and the action the
/// interact key does.
type Hand<'w> = (
    Res<'w, ButtonInput<KeyCode>>,
    Res<'w, Persistent<Settings>>,
    Option<ResMut<'w, Pack>>,
    ResMut<'w, Interact>,
);

/// A thrown tool in the world, with its place and what it hangs from.
type Thrown<'a> = (Entity, &'a Spear, &'a GlobalTransform, Option<&'a ChildOf>);

/// Returns how far from the line of sight from `eye` along the unit `dir` the shaft of `spear`
/// placed at `placed` passes, at its middle, when that lies ahead of the eyes.
fn off(spear: &Spear, placed: &GlobalTransform, (eye, dir): (Vec3, Vec3)) -> Option<f32> {
    let length = spear.item.build.grip + spear.item.build.reach();
    let middle = placed.translation() - placed.rotation() * Vec3::Y * (length / 2.0);
    let along = (middle - eye).dot(dir);
    (along > 0.0).then(|| (middle - eye - dir * along).length())
}

/// Returns the action the robot at `at` looking from `eye` along the unit `dir` may do to `thrown`,
/// and how near the line of sight it is: pull out one stuck in an animal within [`PULL`], take one
/// lying on the ground or pull out one planted in it within [`TAKE_REACH`] and [`TAKE_AIM`].
fn verb(
    thrown: &Thrown,
    at: Vec3,
    (eye, dir): (Vec3, Vec3),
    creatures: &Query<(), With<Creature>>,
) -> Option<(Verb, f32)> {
    let (_, spear, placed, of) = thrown;
    let near = placed.translation().distance(at);
    let animal = of.is_some_and(|of| creatures.contains(of.parent()));
    match (spear.rest, animal) {
        (Rest::Flying, _) => None,
        (Rest::Stuck, true) => (near < PULL).then_some((Verb::Pull, 0.0)),
        (rest, false) => {
            let miss = off(spear, placed, (eye, dir)).filter(|miss| *miss <= TAKE_AIM)?;
            let verb = [Verb::Take, Verb::Pull][usize::from(rest == Rest::Stuck)];
            (near <= TAKE_REACH).then_some((verb, miss))
        }
        (Rest::Lying, true) => None,
    }
}

/// Returns the name of the tool of `spear` for the prompt, kept in `names` so each is made once:
/// its head material and its kind, as the pack calls it.
fn named(names: &mut Vec<(u32, &'static str)>, spear: &Spear) -> &'static str {
    let id = spear.item.id;
    if let Some((_, name)) = names.iter().find(|(known, _)| *known == id) {
        return *name;
    }
    let name = spear
        .lent
        .as_ref()
        .map_or_else(|| "javelin".to_string(), |lent| lent.0.name());
    let name: &'static str = Box::leak(name.into_boxed_str());
    names.push((id, name));
    name
}

/// Offers to take the thrown tool the robot looks at or stands by, and gives it back to the robot
/// when the interact key goes down while the prompt names that take.
pub fn pull(
    mut commands: Commands,
    (keys, settings, mut pack, mut interact): Hand,
    (robot, camera, mut names): (
        Robot,
        Query<&GlobalTransform, With<GameCamera>>,
        Local<Vec<(u32, &'static str)>>,
    ),
    spears: Query<(Entity, &Spear, &GlobalTransform, Option<&ChildOf>)>,
    (players, full, creatures): Reach,
) {
    let (Some((body, player)), Ok(eye)) = (robot.pair(), camera.single()) else {
        return;
    };
    let Ok(at) = players.get(player) else { return };
    let free = !full.contains(body);
    let sight = (eye.translation(), eye.forward().as_vec3());
    let Some((verb, (entity, spear, _, _))) = spears
        .iter()
        .filter(|(_, spear, _, _)| free || spear.lent.is_some())
        .filter_map(|thrown| verb(&thrown, at.translation, sight, &creatures).map(|v| (v, thrown)))
        .min_by(|a, b| a.0.1.total_cmp(&b.0.1))
        .map(|((verb, _), thrown)| (verb, thrown))
    else {
        return;
    };
    let name = named(&mut names, spear);
    interact.offer(verb, name);
    let mine = interact.chosen().is_some_and(|offer| offer.thing == name);
    if mine && interact.pressed(verb, &keys, settings.controls.keys.ship.board) {
        take(&mut commands, body, spear, pack.as_deref_mut());
        commands.entity(entity).despawn();
    }
}

/// Gives the item of `spear` back to `body`: into the hotbar slot it was lent from, or into the
/// hands when nothing lent it.
pub fn take(commands: &mut Commands, body: Entity, spear: &Spear, pack: Option<&mut Pack>) {
    match (spear.lent.clone(), pack) {
        (Some(lent), Some(pack)) => pack.restore(lent),
        _ => {
            commands.entity(body).insert(spear.item.clone());
        }
    }
}
