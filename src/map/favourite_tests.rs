//! Tests of the favourites: drawn once the universe exists, and carried by the world and the
//! record a server keeps.

use abstractions_net::log::{Changes, PlayerId};
use abstractions_net::wire::Welcome;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Favourites;
use crate::menus::{MarkerKind, Markers};
use crate::net::{record_of, world_of};
use crate::saves::{ActiveWorld, WorldEntry, WorldSave};
use crate::universe::Universe;

/// Favourites set before the universe exists are drawn once it does.
#[test]
fn favourites_set_before_the_universe_are_drawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Markers>()
        .init_resource::<Favourites>()
        .add_systems(Update, super::fill);
    app.update();
    let home = Universe::new(3).home();
    app.world_mut()
        .resource_mut::<Favourites>()
        .save(home, DVec3::X);
    app.update();
    assert!(app.world().resource::<Markers>().0.is_empty());
    app.world_mut().insert_resource(Universe::new(3));
    app.update();
    let markers = &app.world().resource::<Markers>().0;
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].kind, MarkerKind::Favourite);
}

/// The favourites of the active world load, go back into it when they change, and ride the
/// record a server keeps to the world of the next welcome.
#[test]
fn favourites_cross_the_world_and_the_record() {
    let home = Universe::new(3).home();
    let mut marks = Favourites::default();
    marks.save(home, DVec3::X);
    let save = WorldSave {
        seed: 3,
        marks: marks.0.clone(),
        ..WorldSave::default()
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Favourites>()
        .insert_resource(ActiveWorld(WorldEntry {
            id: "isle".to_string(),
            save,
        }))
        .add_systems(Update, (super::load, super::store).chain());
    app.update();
    assert_eq!(*app.world().resource::<Favourites>(), marks);
    let index = app
        .world_mut()
        .resource_mut::<Favourites>()
        .save(home, DVec3::Y);
    app.world_mut()
        .resource_mut::<Favourites>()
        .rename(index, "Cove");
    app.update();
    let saved = app.world().resource::<ActiveWorld>().0.save.clone();
    assert_eq!(saved.marks, app.world().resource::<Favourites>().0);
    assert_eq!(saved.marks[1].name, "Cove");
    let welcome = Welcome {
        you: PlayerId(3),
        seed: 3,
        clock: 0.0,
        changes: Changes::default(),
        atlas: String::new(),
        record: Some(record_of(&saved)),
        players: Vec::new(),
        explored: Vec::new(),
        waypoints: Vec::new(),
    };
    assert_eq!(world_of(&welcome, "Isle").save.marks, saved.marks);
}
