//! Tests of the hard end of the loading screen: the steps in order, and a world that never
//! gets ready started anyway.

use std::time::Duration;

use bevy::prelude::*;

use super::stall::{LAND_AFTER, LANDED, START_AFTER, STARTED, Stall, Step, watch};
use crate::saves::{ActiveWorld, Place, WorldEntry, WorldSave};
use crate::world::{Resettled, WorldReady};

/// Returns an app that only counts the loading time, `seconds` into a world that is not ready.
fn loading(seconds: f32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(Stall {
            seconds,
            landed: false,
        })
        .insert_resource(WorldReady(false))
        .add_systems(Update, watch);
    app
}

/// Runs frames until `done` holds, for at most two seconds of them.
fn run(app: &mut App, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..1000 {
        app.update();
        if done(app.world()) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    false
}

/// The body lands once after a minute of loading and the game starts after two, whatever the
/// frames were like; a frame that blocked for long counts for a quarter of a second.
#[test]
fn the_wait_ends_in_two_steps() {
    let mut stall = Stall::default();
    assert_eq!(stall.after(30.0), Step::Wait);
    let mut steps = Vec::new();
    for frame in 1..=480 {
        match stall.after(0.25) {
            Step::Wait => {}
            step => steps.push((frame, step)),
        }
    }
    assert_eq!(steps[0], (239, Step::Land));
    assert_eq!(steps[1], (479, Step::Start));
    assert_eq!(steps[2..], [(480, Step::Start)]);
    const { assert!(LAND_AFTER < START_AFTER) };
}

/// A world that never gets ready is flagged ready once the time is up, so the loading screen
/// ends, and the player is told. A world that is ready is left alone.
#[test]
fn a_world_that_never_loads_starts_anyway() {
    let mut app = loading(START_AFTER - 0.001);
    assert!(run(&mut app, |world| world.resource::<WorldReady>().0));
    assert_eq!(app.world().resource::<Resettled>(), &Resettled(STARTED));
    let mut ready = loading(0.0);
    ready.insert_resource(WorldReady(true));
    ready.update();
    ready.update();
    assert_eq!(ready.world().resource::<Stall>().seconds, 0.0);
}

/// The body of a resumed world is stood on the ground after a minute and the world is still
/// waited for; a new game, whose body has no saved place, is not moved.
#[test]
fn a_resumed_body_lands_before_the_game_gives_up() {
    let place = Place {
        dir: [0.0, 1.0, 0.0],
        altitude: 6560.18,
        ..Place::default()
    };
    let save = WorldSave {
        place: Some(place),
        ..WorldSave::default()
    };
    let id = "w".to_string();
    let mut app = loading(LAND_AFTER - 0.001);
    app.insert_resource(ActiveWorld(WorldEntry { id, save }));
    assert!(run(&mut app, |world| world.contains_resource::<Resettled>()));
    assert_eq!(app.world().resource::<Resettled>(), &Resettled(LANDED));
    assert!(!app.world().resource::<WorldReady>().0);
    let mut fresh = loading(LAND_AFTER - 0.001);
    assert!(run(&mut fresh, |world| world.resource::<Stall>().landed));
    assert!(!fresh.world().contains_resource::<Resettled>());
}
