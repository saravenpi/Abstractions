//! Tests of the end of pins: with the world, and with the session.

use std::sync::{Arc, Mutex};

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::Places;
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::net::{Link, Role, Script, Source, Wire};
use crate::saves::Pin;
use crate::screens::Screen;
use crate::universe::Universe;

/// Returns a pin on the home planet of seed 3.
fn spot() -> Pin {
    Pin {
        name: "Camp".to_string(),
        body: Universe::new(3).home(),
        dir: DVec3::X.to_array(),
        height: None,
    }
}

/// Returns a waypoint marker of `owner`.
fn marker(owner: u32) -> Marker {
    Marker {
        kind: MarkerKind::Waypoint,
        place: MarkerPlace::Anchor(DVec3::ZERO),
        color: Color::WHITE,
        label: String::new(),
        owner,
    }
}

/// Returns a headless app in play with this player's pin, player 4's active pin and their markers.
fn playing() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(Places {
            me: 2,
            pins: vec![spot()],
            active: Some(0),
            friends: [(4, spot())].into(),
            ..Places::default()
        })
        .insert_resource(Markers(vec![marker(0), marker(4)]))
        .insert_state(Screen::Gameplay)
        .add_plugins(super::plugin);
    app.update();
    app
}

/// Back on the title screen no pin is left, so the key of the next world drops the first.
#[test]
fn pins_end_with_the_world() {
    let mut app = playing();
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Title);
    app.update();
    assert_eq!(*app.world().resource::<Places>(), Places::default());
    assert!(app.world().resource::<Markers>().0.is_empty());
    let mut places = app.world_mut().resource_mut::<Places>();
    assert_eq!(places.drop(spot().body, DVec3::X), Some(0));
}

/// When the session ends the others' pins go, and this player's own stay.
#[test]
fn the_others_pins_end_with_the_session() {
    let mut app = playing();
    let script: Script = Arc::new(Mutex::new(Wire::default()));
    let link = Link::over(Source::Script(script), Role::Host);
    app.world_mut().insert_resource(link);
    app.update();
    app.world_mut().remove_resource::<Link>();
    app.update();
    let places = app.world().resource::<Places>();
    assert_eq!(
        (places.pins.clone(), places.active),
        (vec![spot()], Some(0))
    );
    assert!(places.friends.is_empty());
    assert_eq!(places.me, 0);
    assert_eq!(app.world().resource::<Markers>().0, vec![marker(0)]);
}
