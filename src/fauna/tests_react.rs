//! Tests of reactions: an animal flees a robot that comes too near, once its senses notice it, and
//! walks back to its herd once the robot is gone.

use bevy::math::{DVec3, Vec3};
use bevy::prelude::*;

use super::beast::Beast;
use super::gate::Spec;
use super::herds::{Herds, Land, member};
use super::react::Act;
use super::rig::app;
use super::stage::{put, run, staying};
use super::want::Lead;
use crate::world::Terrain;

/// A point in time as the C library gives it.
#[repr(C)]
#[derive(Default)]
struct Timespec {
    /// Whole seconds.
    seconds: i64,
    /// Nanoseconds past the whole seconds.
    nanos: i64,
}

unsafe extern "C" {
    /// Reads the clock `clock` into `time`.
    fn clock_gettime(clock: i32, time: *mut Timespec) -> i32;
}

/// The clock that counts the processor time of the calling thread on Linux.
#[cfg(target_os = "linux")]
const THREAD_CLOCK: i32 = 3;

/// The clock that counts the processor time of the calling thread on macOS.
#[cfg(target_os = "macos")]
const THREAD_CLOCK: i32 = 16;

/// Returns the seconds of processor time the calling thread has used.
pub fn busy() -> f64 {
    let mut time = Timespec::default();
    let failed = unsafe { clock_gettime(THREAD_CLOCK, &mut time) };
    assert_eq!(failed, 0);
    time.seconds as f64 + time.nanos as f64 * 1.0e-9
}

/// Returns the animal of `app` led by `key`.
fn find(app: &mut App, key: (u64, u8)) -> Option<Beast> {
    staying(app).into_iter().find(|b| b.lead.key() == key)
}

/// Returns the metres along the ground in its frame from `beast` to its place in its herd now.
fn off_place(app: &mut App, beast: &Beast) -> f32 {
    let world = app.world_mut();
    let globe = world.resource::<Terrain>().globe().clone();
    let (Some(life), Some(planet)) = (globe.life(), world.resource::<Spec>().planet.clone()) else {
        return f32::MAX;
    };
    let time = world.resource::<crate::space::Clock>().seconds;
    let land = Land {
        globe: &globe,
        life,
        planet: &planet,
    };
    let id = beast.lead.key().0;
    let herd = world.resource_mut::<Herds>().herd(land, id, time);
    let place = herd.and_then(|h| member(&h, beast.lead.key().1, time, globe.radius()));
    place.map_or(f32::MAX, |(_, dir)| {
        let at = beast.spot.local(dir * globe.radius());
        (at - beast.motion.root.at).with_y(0.0).length()
    })
}

/// A robot a metre and a half from an animal makes it flee, and once the robot is far it calms down
/// and walks back to its place in the herd.
#[test]
fn animals_flee_a_near_robot_and_come_back() {
    let (mut app, _) = app(true);
    run(&mut app, 30.0, 0.1);
    let herded = staying(&mut app)
        .into_iter()
        .find(|b| matches!(b.lead, Lead::Herd { .. }));
    let beast = herded.expect("a herd member");
    let key = beast.lead.key();
    put(
        &mut app,
        beast
            .spot
            .planet(beast.motion.root.at + Vec3::new(1.5, 1.0, 0.0)),
    );
    run(&mut app, 1.5, 0.05);
    let scared = find(&mut app, key).expect("the animal stays drawn");
    assert_eq!(scared.mood.act(), Act::Flee);
    let fled = off_place(&mut app, &scared);
    let far: DVec3 = scared
        .spot
        .planet(scared.motion.root.at + Vec3::new(0.0, 1.0, 110.0));
    put(&mut app, far);
    run(&mut app, 60.0, 0.1);
    let calm = find(&mut app, key).expect("the animal stays drawn");
    let back = off_place(&mut app, &calm);
    println!("fled {fled:.1} m from its place, {back:.1} m once calm");
    assert_eq!(calm.mood.act(), Act::Graze);
    assert!(back < 3.0, "{back} m from its place in the herd");
}
