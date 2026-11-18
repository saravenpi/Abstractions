//! Computer tests: the climate of the card in an app without a window, its line and its map
//! painted once per planet and step of the year.

use std::time::{Duration, Instant};

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::inset::{ClimateLine, ClimateMap, Inset, STEPS, collect, line, map, plan};
use super::tests_climate::home;
use super::*;
use crate::space::Clock;
use crate::universe::PlanetSpec;

/// Returns an app with a board of two charted planets and one that is not, the card's line
/// and map, and the clock at the start.
fn card() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).init_resource::<Inset>();
    let first = PlanetSpec {
        orbit_phase: 0.0,
        ..home()
    };
    let mut second = first.clone();
    second.id.index += 1;
    let board = Board {
        worlds: vec![Some(first), Some(second), None],
        ..default()
    };
    app.insert_resource(board).insert_resource(Clock::default());
    app.world_mut().spawn(line());
    app.world_mut().spawn(map());
    app
}

/// Runs `plan` and returns whether it started painting a map, then waits for the map.
fn step(app: &mut App) -> bool {
    app.world_mut().run_system_once(plan).unwrap();
    let started = app.world().resource::<Inset>().job.is_some();
    let until = Instant::now() + Duration::from_secs(20);
    while app.world().resource::<Inset>().job.is_some() && Instant::now() < until {
        app.world_mut().run_system_once(collect).unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    started
}

/// Returns what the card shows: whether the map is drawn and the climate line.
fn shown(app: &mut App) -> (bool, String) {
    let world = app.world_mut();
    let drawn = world
        .query_filtered::<&Node, With<ClimateMap>>()
        .single(world)
        .unwrap()
        .display;
    let text = world
        .query_filtered::<&Text, With<ClimateLine>>()
        .single(world)
        .unwrap();
    (drawn == Display::Flex, text.0.clone())
}

/// The map is painted when the planet under the cursor or the step of its year changes, and
/// never again while both stay; a planet whose system was not visited shows nothing.
#[test]
fn the_climate_map_is_painted_once_per_planet_and_step() {
    let mut app = card();
    assert!(step(&mut app));
    let (drawn, written) = shown(&mut app);
    assert!(drawn && written.starts_with("northern "), "{written}");
    assert!(!step(&mut app) && !step(&mut app));
    let year = app.world().resource::<Board>().worlds[0]
        .as_ref()
        .unwrap()
        .orbit_seconds;
    app.world_mut().resource_mut::<Clock>().seconds += year / f64::from(STEPS);
    assert!(step(&mut app) && !step(&mut app));
    app.world_mut().resource_mut::<Board>().cursor = 1;
    assert!(step(&mut app) && !step(&mut app));
    assert!(shown(&mut app).0);
    app.world_mut().resource_mut::<Board>().cursor = 2;
    assert!(!step(&mut app));
    assert_eq!(shown(&mut app), (false, String::new()));
}
