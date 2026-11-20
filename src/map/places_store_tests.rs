//! Tests of the pins as the world keeps them: loaded from the world and old worlds, drawn as
//! markers, and told to the server.

use bevy::math::DVec3;
use bevy::prelude::*;
use serde::Serialize;

use super::Places;
use crate::map::link::{WaypointChanged, marks_of, marks_text};
use crate::menus::{MarkerKind, Markers};
use crate::saves::{ActiveWorld, Pin, WorldEntry, WorldSave};
use crate::universe::{BodyId, Universe};

/// Returns two pins on the home planet of seed 3.
fn kept() -> Vec<Pin> {
    let body = Universe::new(3).home();
    let pin = |name: &str, dir: DVec3| Pin {
        name: name.to_string(),
        body,
        dir: dir.to_array(),
    };
    vec![pin("Camp", DVec3::X), pin("Cove", DVec3::Y)]
}

/// A world with pins loads them, the active pin is none, and a change goes back into the world.
#[test]
fn pins_load_and_store_through_the_world() {
    let (home, pins) = (Universe::new(3).home(), kept());
    let save = WorldSave {
        seed: 3,
        marks: pins.clone(),
        ..WorldSave::default()
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Places>()
        .insert_resource(ActiveWorld(WorldEntry {
            id: "isle".to_string(),
            save,
        }))
        .add_systems(Update, (super::load, super::store).chain());
    app.update();
    assert_eq!(app.world().resource::<Places>().pins, pins);
    assert_eq!(app.world().resource::<Places>().active, None);
    let mut places = app.world_mut().resource_mut::<Places>();
    places.drop(home, DVec3::Z);
    places.rename(2, "Dune");
    app.update();
    let saved = &app.world().resource::<ActiveWorld>().0.save;
    assert_eq!(saved.marks, app.world().resource::<Places>().pins);
    assert_eq!(saved.marks[2].name, "Dune");
}

/// What an old world wrote for a favourite: the same record a pin is.
#[derive(Serialize)]
struct Old {
    name: String,
    body: BodyId,
    dir: [f64; 3],
}

/// The records of old worlds and of the server record load as pins, and the pins write back.
#[test]
fn old_favourites_load_as_pins() {
    let body = Universe::new(3).home();
    let old = vec![Old {
        name: "Place 1".to_string(),
        body,
        dir: DVec3::X.to_array(),
    }];
    #[derive(Serialize)]
    struct Record {
        mark: Vec<Old>,
    }
    let text = toml::to_string(&Record { mark: old }).unwrap();
    let pins = marks_of(&text);
    assert_eq!(pins.len(), 1);
    assert_eq!((pins[0].name.as_str(), pins[0].body), ("Place 1", body));
    assert_eq!(marks_of(&marks_text(&pins)), pins);
    assert_eq!((marks_text(&[]), marks_of("")), (String::new(), Vec::new()));
}

/// Returns a headless app that draws the pins as markers.
fn drawn() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Markers>()
        .init_resource::<Places>()
        .add_systems(Update, super::fill);
    app
}

/// Pins set before the universe exists are drawn once it does, the active one as a waypoint
/// marker and the others as favourite markers, a friend's pin as a waypoint of its owner.
#[test]
fn pins_are_drawn_once_the_universe_exists() {
    let mut app = drawn();
    let home = Universe::new(3).home();
    let mut places = app.world_mut().resource_mut::<Places>();
    places.drop(home, DVec3::X);
    places.drop(home, DVec3::Y);
    places.set_active(Some(1));
    places.friends.insert(
        4,
        Pin {
            name: "Mine".to_string(),
            body: home,
            dir: DVec3::Z.to_array(),
        },
    );
    app.update();
    assert!(app.world().resource::<Markers>().0.is_empty());
    app.world_mut().insert_resource(Universe::new(3));
    app.update();
    let markers = &app.world().resource::<Markers>().0;
    let kinds: Vec<(MarkerKind, u32)> = markers.iter().map(|m| (m.kind, m.owner)).collect();
    assert_eq!(
        kinds,
        [
            (MarkerKind::Favourite, 0),
            (MarkerKind::Waypoint, 0),
            (MarkerKind::Waypoint, 4)
        ]
    );
    assert_eq!(markers[1].label, "Pin 2");
}

/// Messages the app collected.
#[derive(Resource, Default)]
struct Sent(Vec<Option<Pin>>);

/// Collects the changes of the active pin.
fn collect(mut read: MessageReader<WaypointChanged>, mut sent: ResMut<Sent>) {
    sent.0.extend(read.read().map(|change| change.0.clone()));
}

/// The server hears of the active pin when it is set or cleared, and of nothing else.
#[test]
fn the_server_hears_of_the_active_pin_only() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Places>()
        .init_resource::<Sent>()
        .add_message::<WaypointChanged>()
        .add_systems(Update, (super::announce, collect).chain());
    app.update();
    let home = Universe::new(3).home();
    app.world_mut()
        .resource_mut::<Places>()
        .drop(home, DVec3::X);
    app.update();
    assert!(app.world().resource::<Sent>().0.is_empty());
    app.world_mut().resource_mut::<Places>().set_active(Some(0));
    app.update();
    app.world_mut().resource_mut::<Places>().rename(0, "Camp");
    app.update();
    app.world_mut().resource_mut::<Places>().set_active(None);
    app.update();
    let sent = &app.world().resource::<Sent>().0;
    assert_eq!(sent.len(), 3);
    assert_eq!(sent[0].as_ref().map(|pin| pin.name.as_str()), Some("Pin 1"));
    assert_eq!(sent[1].as_ref().map(|pin| pin.name.as_str()), Some("Camp"));
    assert_eq!(sent[2], None);
}
