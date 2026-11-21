//! Tests of closing in, run through the walk and the attacks as the game runs them: an animal that
//! goes for the robot runs up to where its attack reaches and lands a blow, and one that stands its
//! ground goes for a robot that comes within its guard, never for one far off.

use std::time::Duration;

use bevy::ecs::message::Messages;
use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::kinds::Kind;
use super::limbs::Broken;
use super::pounce::attack;
use crate::audio::Impact;
use crate::avatar::blow::Blow;
use crate::fauna::beast::Beast;
use crate::fauna::rig::app_of;
use crate::fauna::senses::{Focus, Stance};
use crate::fauna::stage::{put, run};
use crate::fauna::walk::walk;
use crate::fauna::want::Lead;
use crate::life::Species;
use crate::life::body::Locomotion;
use crate::space::Clock;
use crate::world::Terrain;

/// Returns true for a walker that bites.
fn biter(species: &Species) -> bool {
    species.body.locomotion == Locomotion::Walk && Kind::Bite.can(species, Broken::default())
}

/// Returns an app with a herd of walkers that bite drawn around the robot, the key of one of them,
/// its bite reach, and the planet position `metres` straight ahead of it, where the robot now stands.
fn meet(metres: impl Fn(f32) -> f32) -> (App, (u64, u8), f32, DVec3) {
    let (mut app, herd) = app_of(true, biter);
    assert!(herd.is_some(), "no herd of walkers that bite");
    app.add_message::<Blow>().add_message::<Impact>();
    run(&mut app, 3.0, 1.0 / 30.0);
    let world = app.world_mut();
    let life = world.resource::<Terrain>().globe().life().cloned();
    let life = life.expect("the rig's planet lives");
    let mut beasts = world.query::<&Beast>();
    let found = beasts.iter(world).find_map(|b| {
        let species = life.species.get(usize::from(b.index))?;
        let herded = matches!(b.lead, Lead::Herd { .. }) && !b.fade.leaving();
        let reach = Kind::Bite.reach(&species.body);
        let way = (b.motion.root.turn * Vec3::NEG_Z)
            .with_y(0.0)
            .normalize_or(Vec3::NEG_Z);
        let ahead = b.motion.root.at + way * metres(reach) + Vec3::Y;
        (herded && biter(species)).then(|| (b.lead.key(), reach, b.spot.planet(ahead)))
    });
    let (key, reach, at) = found.expect("a drawn walker that bites");
    put(&mut app, at);
    (app, key, reach, at)
}

/// Runs the attacks and the walk for `seconds` in frames of a thirtieth of a second, the animal `key`
/// held in `stance` towards the robot at planet position `robot`. Returns the least metres between
/// them along the ground and the blows that landed.
fn hold(
    app: &mut App,
    (key, stance): ((u64, u8), Stance),
    robot: DVec3,
    seconds: f32,
) -> (f32, usize) {
    let mut least = f32::MAX;
    for _ in 0..(seconds * 30.0) as u32 {
        let world = app.world_mut();
        let dt = 1.0 / 30.0;
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        world.resource_mut::<Clock>().seconds += f64::from(dt);
        let mut beasts = world.query::<&mut Beast>();
        for mut beast in beasts.iter_mut(world).filter(|b| b.lead.key() == key) {
            let at = beast.spot.local(robot);
            beast.wits.stance = stance;
            beast.wits.focus = Some(Focus { at, aware: 1.0 });
            least = least.min((at - beast.motion.root.at).with_y(0.0).length());
        }
        world.run_system_once(attack).expect("the attacks run");
        world.run_system_once(walk).expect("the walk runs");
    }
    let blows = app
        .world_mut()
        .resource_mut::<Messages<Blow>>()
        .drain()
        .count();
    (least, blows)
}

/// An animal that goes for a robot 9 m ahead runs up to it, within the reach of its bite, and lands a
/// blow within six seconds.
#[test]
fn an_attacker_runs_in_and_strikes() {
    let (mut app, key, reach, robot) = meet(|_| 9.0);
    let (least, blows) = hold(&mut app, (key, Stance::Attack), robot, 6.0);
    println!("closed to {least:.2} m, bite reach {reach:.2} m, {blows} blows");
    assert!(
        least <= reach * super::tune::SLACK,
        "stopped {least} m off, its bite reaches {reach} m"
    );
    assert!(blows > 0, "no blow landed");
}

/// An animal that stands its ground faces a robot 25 m off and stays put, but goes for one that comes
/// within two metres past its bite, and lands a blow.
#[test]
fn a_standing_animal_goes_for_a_robot_too_near() {
    let (mut app, key, _, far) = meet(|_| 25.0);
    let (least, blows) = hold(&mut app, (key, Stance::Threaten), far, 3.0);
    assert!(least > 23.0 && blows == 0, "went for a robot {least} m off");
    let (mut app, key, reach, near) = meet(|reach| reach + 2.0);
    let (least, blows) = hold(&mut app, (key, Stance::Threaten), near, 4.0);
    println!("closed to {least:.2} m, bite reach {reach:.2} m, {blows} blows");
    assert!(blows > 0, "no blow on a robot {} m off", reach + 2.0);
}
