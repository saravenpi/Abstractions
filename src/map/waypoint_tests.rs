//! Tests of the end of waypoints: with the world, and with the session.

use std::sync::{Arc, Mutex};

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::{Waypoint, Waypoints, toggle};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::net::{Link, Role, Script, Source, Wire};
use crate::screens::Screen;
use crate::universe::Universe;

/// Returns a waypoint on the home planet of seed 3.
fn spot() -> Waypoint {
    Waypoint {
        body: Universe::new(3).home(),
        dir: DVec3::X,
        name: String::new(),
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

/// Returns a headless app in play with this player's waypoint, player 4's and their markers.
fn playing() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(Waypoints {
            me: 2,
            mine: Some(spot()),
            friends: [(4, spot())].into(),
        })
        .insert_resource(Markers(vec![marker(0), marker(4)]))
        .insert_state(Screen::Gameplay)
        .add_plugins(super::plugin);
    app.update();
    app
}

/// Back on the title screen no waypoint is left, so the first key of the next world sets one.
#[test]
fn waypoints_end_with_the_world() {
    let mut app = playing();
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Title);
    app.update();
    assert_eq!(*app.world().resource::<Waypoints>(), Waypoints::default());
    assert!(app.world().resource::<Markers>().0.is_empty());
    let mut waypoints = app.world_mut().resource_mut::<Waypoints>();
    let set = toggle(&mut waypoints, Some((spot().body, DVec3::X)));
    assert!(set.is_some_and(|change| change.0.is_some()));
}

/// When the session ends the others' waypoints go, and this player's own stays.
#[test]
fn the_others_waypoints_end_with_the_session() {
    let mut app = playing();
    let script: Script = Arc::new(Mutex::new(Wire::default()));
    let link = Link::over(Source::Script(script), Role::Host);
    app.world_mut().insert_resource(link);
    app.update();
    app.world_mut().remove_resource::<Link>();
    app.update();
    let waypoints = app.world().resource::<Waypoints>();
    assert_eq!(waypoints.mine, Some(spot()));
    assert!(waypoints.friends.is_empty());
    assert_eq!(waypoints.me, 0);
    assert_eq!(app.world().resource::<Markers>().0, vec![marker(0)]);
}

/// Waypoints a welcome sets before the universe exists are drawn once it does.
#[test]
fn waypoints_set_before_the_universe_are_drawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Markers>()
        .init_resource::<Waypoints>()
        .add_systems(Update, super::fill);
    app.update();
    app.world_mut()
        .resource_mut::<Waypoints>()
        .friends
        .insert(4, spot());
    app.update();
    assert!(app.world().resource::<Markers>().0.is_empty());
    app.world_mut().insert_resource(Universe::new(3));
    app.update();
    let markers = &app.world().resource::<Markers>().0;
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].owner, 4);
}
