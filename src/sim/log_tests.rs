//! Tests of the log of first visits: what a visit appends, and that the atlas a world brings
//! is never logged, whether it is loaded first or after another world.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::log::plugin;
use super::{Change, Changes, PlayerId};
use crate::computer::{Atlas, Target};
use crate::menus::Menu;
use crate::saves::{ActiveWorld, Starter, WorldEntry, WorldSave};
use crate::screens::Screen;
use crate::space::Clock;
use crate::universe::{BodyId, Universe};

/// Returns an app on the title screen with the logger, a new game's atlas and the clock at
/// `seconds`.
fn app(seconds: f64) -> App {
    let mut app = App::new();
    app.add_plugins((StatesPlugin, plugin))
        .init_state::<Screen>()
        .init_resource::<Changes>()
        .insert_resource(Atlas::new(&Universe::new(9)))
        .insert_resource(Clock { seconds });
    app.update();
    app
}

/// The atlas of a new game logs nothing; a first visit appends one entry at the universe
/// time, a second visit of the same planet none.
#[test]
fn a_first_visit_appends_one_entry() {
    let mut app = app(1234.5);
    assert!(app.world().resource::<Changes>().entries.is_empty());
    let home = Universe::new(9).home();
    let other = BodyId {
        index: home.index + 1,
        ..home
    };
    for _ in 0..2 {
        let mut atlas = app.world_mut().resource_mut::<Atlas>();
        atlas.visit(Target::Body(other));
        app.update();
    }
    let entries = &app.world().resource::<Changes>().entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].change, Change::Visited(other));
    assert_eq!(
        (entries[0].player, entries[0].time),
        (PlayerId::LOCAL, 1234.5)
    );
}

/// A world loaded in place of another brings an atlas whose planets are not logged again.
#[test]
fn a_loaded_atlas_is_not_logged() {
    let mut app = app(0.0);
    let mut atlas = Atlas::new(&Universe::new(9));
    let home = atlas.bodies[0];
    atlas.visit(Target::Body(BodyId { moon: 1, ..home }));
    app.insert_resource(atlas);
    let world = app.world_mut();
    world
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Loading);
    app.update();
    app.update();
    assert!(app.world().resource::<Changes>().entries.is_empty());
}

/// Returns a world of seed 9 saved as `id` that has visited `moons` moons of its home.
fn world(id: &str, moons: u8) -> WorldEntry {
    let mut atlas = Atlas::new(&Universe::new(9));
    let home = atlas.bodies[0];
    for moon in 1..=moons {
        atlas.visit(Target::Body(BodyId { moon, ..home }));
    }
    let save = WorldSave {
        seed: 9,
        atlas: Some(atlas),
        ..WorldSave::default()
    };
    WorldEntry {
        id: id.to_string(),
        save,
    }
}

/// Starts world A on the second frame and world B on the fifth, from `Update` as the menus do.
fn start_next(mut frame: Local<u32>, mut starter: Starter) {
    *frame += 1;
    match *frame {
        2 => starter.start(world("a", 1)),
        5 => starter.start(world("b", 3)),
        _ => {}
    }
}

/// World A then world B started in the same app, as the menus start them: the longer atlas
/// of B is never compared with the mark of A, so nothing is logged into the log B brought.
#[test]
fn a_world_started_after_another_logs_nothing() {
    let mut app = app(0.0);
    app.init_state::<Menu>().add_systems(Update, start_next);
    for _ in 0..8 {
        app.update();
    }
    assert_eq!(app.world().resource::<ActiveWorld>().0.id, "b");
    assert_eq!(app.world().resource::<Atlas>().bodies.len(), 4);
    assert!(app.world().resource::<Changes>().entries.is_empty());
}
