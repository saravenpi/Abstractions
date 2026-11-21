//! Tests of an attack run as the game runs it: an animal that goes for the robot winds up and lands
//! one blow on it, also when the robot of another player stands in the world.

use std::time::Duration;

use bevy::ecs::message::Messages;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::kinds::Kind;
use super::limbs::Broken;
use super::pounce::attack;
use super::tune::SPREAD;
use super::{Attacking, Dead};
use crate::audio::Impact;
use crate::avatar::blow::Blow;
use crate::avatar::{Avatar, AvatarPose};
use crate::fauna::beast::Beast;
use crate::fauna::rig::app_of;
use crate::fauna::senses::{Focus, Stance};
use crate::fauna::stage::{put, run};
use crate::fauna::want::Lead;
use crate::life::Species;
use crate::player::Player;
use crate::space::Anchor;
use crate::world::Terrain;

/// Returns true when a body of `species` can bite.
fn bites(species: &Species) -> bool {
    Kind::Bite.can(species, Broken::default())
}

/// Makes the first living animal of `app` that can bite go for a point within its bite, straight
/// ahead of it, and returns that point and the place of the animal, both as render positions.
fn provoke(app: &mut App) -> Option<(Vec3, Vec3)> {
    let world = app.world_mut();
    let (terrain, anchor) = (
        world.resource::<Terrain>().clone(),
        *world.resource::<Anchor>(),
    );
    let life = terrain.globe().life()?;
    let mut beasts = world.query_filtered::<&mut Beast, Without<Dead>>();
    let mut found = None;
    for mut beast in beasts.iter_mut(world) {
        let species = life.species.get(usize::from(beast.index));
        let Some(species) = species.filter(|s| bites(s) && matches!(beast.lead, Lead::Herd { .. }))
        else {
            continue;
        };
        let ahead = (beast.motion.root.turn * Vec3::NEG_Z).with_y(0.0);
        let reach = 0.8 * Kind::Bite.reach(&species.body);
        let at = beast.motion.root.at + ahead.normalize_or(Vec3::NEG_Z) * reach;
        beast.wits.stance = Stance::Attack;
        beast.wits.focus = Some(Focus { at, aware: 1.0 });
        let render = |local: Vec3| anchor.to_render(beast.spot.planet(local));
        found = Some((render(at), render(beast.motion.root.at)));
        break;
    }
    found
}

/// Runs the attacks alone for `seconds`, in frames of a thirtieth of a second.
fn fight(app: &mut App, seconds: f32) {
    for _ in 0..(seconds * 30.0) as u32 {
        let world = app.world_mut();
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 30.0));
        world.run_system_once(attack).expect("the attacks run");
    }
}

/// An animal that goes for the local robot, in reach, winds up and lands its blow on it. The
/// robot of another player, a body with no player of this machine, stands nearer the animal: the
/// attack still runs, and the blow lands on the local robot, the one this machine answers for.
#[test]
fn an_animal_strikes_the_local_robot_with_another_player_near() {
    let (mut app, herd) = app_of(true, bites);
    assert!(herd.is_some(), "no herd of a species that bites");
    app.add_message::<Blow>().add_message::<Impact>();
    run(&mut app, 3.0, 1.0 / 30.0);
    let (target, animal) = provoke(&mut app).expect("a living animal that bites");
    let anchor = *app.world().resource::<Anchor>();
    put(&mut app, anchor.to_frame(target));
    let other = Transform::from_translation(animal.lerp(target, 0.3));
    let world = app.world_mut();
    world.spawn((Avatar, AvatarPose::default(), other));
    let mut players = world.query_filtered::<(), With<Player>>();
    assert_eq!(players.iter(world).count(), 1);
    fight(&mut app, 0.2);
    let mut winding = app.world_mut().query::<&Attacking>();
    assert!(winding.iter(app.world()).count() >= 1, "no attack began");
    fight(&mut app, 4.0);
    let world = app.world_mut();
    let blows: Vec<Blow> = world.resource_mut::<Messages<Blow>>().drain().collect();
    assert!(!blows.is_empty(), "no blow landed");
    for blow in &blows {
        assert!(blow.joules > 0.0 && blow.momentum > 0.0, "{blow:?}");
        assert!(blow.at.distance(target) <= SPREAD + 0.05, "{blow:?}");
    }
}
