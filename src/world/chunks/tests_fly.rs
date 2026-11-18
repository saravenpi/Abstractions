//! Headless flyovers: the focus flies over a planet at a fixed 60 Hz step while the whole
//! streaming chain runs, and a probe notes what a player would see load.

mod climb;
mod fast;
mod gaps;
mod probe;
mod rig;
mod route;
mod row;
mod still;
mod table;

use std::time::{Duration, Instant};

use bevy::math::DVec3;
use bevy::prelude::*;

use self::gaps::Eye;
use self::probe::Probe;
use self::rig::{Laps, Slow, rig};
use self::route::Route;
use self::row::Row;
use super::Chunks;
use super::budget::{Budget, COVERED, PLAY};
use super::reach::Reach;
use super::tests_reach::resize;
use super::tests_stream::idle;
use crate::space::Anchor;
use crate::world::{SpawnPoint, Terrain};

/// Seconds of one simulated frame.
const STEP: f64 = 1.0 / 60.0;

/// Puts the focus of `app` where `route` is.
fn hold(app: &mut App, route: &Route) {
    let globe = app.world().resource::<Terrain>().globe().clone();
    let anchor = *app.world().resource::<Anchor>();
    let spawn = SpawnPoint(anchor.to_render(route.at(&globe)));
    app.world_mut().insert_resource(spawn);
}

/// Returns true when no loaded chunk rebuilds its terrain or its ground cover and nothing fades.
fn quiet(chunks: &Chunks) -> bool {
    chunks.loaded.values().all(|l| !l.detail.busy()) && chunks.fades.idle()
}

/// Holds the focus where `route` is and runs frames with the generous budget of a loading screen
/// until the streaming has caught up.
fn rest(app: &mut App, route: &Route) {
    app.insert_resource(Budget(COVERED));
    hold(app, route);
    let start = Instant::now();
    app.update();
    while !(idle(app.world()) && quiet(app.world().resource::<Chunks>())) {
        assert!(start.elapsed() < Duration::from_secs(300), "never settled");
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Returns a settled world of `radius` metres at `rings` and a route that starts at its spawn.
fn stage(radius: f64, rings: u32) -> (App, Route) {
    let mut app = rig();
    app.insert_resource(Reach::of(Some(rings)));
    resize(&mut app, radius);
    let anchor = *app.world().resource::<Anchor>();
    let spawn = app.world().resource::<SpawnPoint>().0;
    let up = anchor.to_frame(spawn).normalize();
    let route = Route {
        up,
        heading: up.cross(DVec3::new(0.3, 0.1, 0.9)).normalize(),
        speed: 0.0,
        height: 30.0,
        turns: false,
    };
    rest(&mut app, &route);
    (app, route)
}

/// Moves the focus of `app` one frame along `route` and runs the frame at the pace of a 60 Hz
/// game, so the worker threads get the time they would have, or [`Slow`] times as much.
fn frame(app: &mut App, route: &mut Route) {
    let globe = app.world().resource::<Terrain>().globe().clone();
    route.step(&globe, STEP);
    hold(app, route);
    let start = Instant::now();
    app.update();
    let slow = app
        .world()
        .get_resource::<Slow>()
        .map_or(1.0, |slow| slow.0);
    std::thread::sleep(Duration::from_secs_f64(STEP * slow).saturating_sub(start.elapsed()));
}

/// Seconds a flight runs before it is measured: nothing reaches its speed in one frame, and
/// the streamer needs a moment to see where the focus is going.
const RUN_UP: f64 = 2.0;

/// Flies `route` for `seconds` with the budget of a frame of play, after a run-up at the same
/// speed that is not measured, and returns what showed.
fn fly(app: &mut App, route: &mut Route, seconds: f64) -> Row {
    app.insert_resource(Budget(PLAY));
    (0..(RUN_UP / STEP).round() as usize).for_each(|_| frame(app, route));
    let mut probe = Probe::default();
    let mut row = Row::default();
    for _ in 0..(seconds / STEP).round() as usize {
        frame(app, route);
        let terrain = app.world().resource::<Terrain>().clone();
        let chunks = app.world().resource::<Chunks>();
        let eye = Eye {
            focus: route.at(terrain.globe()),
            heading: route.heading,
            clearance: route.clearance(terrain.globe()),
            terrain: &terrain,
            reach: *app.world().resource::<Reach>(),
            pace: chunks.motion.pace,
        };
        let sample = probe.look(chunks, &eye);
        let millis = app.world().resource::<Laps>().millis;
        row.add(&sample, (eye.fog_end(), seconds), &millis);
    }
    row
}
