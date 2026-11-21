//! Map tests: the Go card follows the picked place, the course and the route.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::deeds::{Button, Deeds, Sentence, deeds, offer, write};
use crate::computer::{
    Act, Ask, Block, Board, Destination, Keys, Line, MapView, NO_PLACE, RouteStatus, Target,
};
use crate::space::{Location, Travel};
use crate::universe::Universe;

/// Returns the names of the default keys.
fn keys() -> Keys {
    Keys {
        board: "F".to_string(),
        rise: "Space".to_string(),
        go: "G".to_string(),
    }
}

/// Returns the question of a card for Mars with the player seated and the route idle, and a
/// trip of about four minutes.
fn base(keys: &Keys) -> Ask<'_> {
    Ask {
        name: "Mars",
        ours: false,
        here: false,
        seated: true,
        status: RouteStatus::Idle,
        eta: Some("about 4 min"),
        keys,
        home: false,
    }
}

/// Without a place the card says how to pick one and has no button. With one it has a button
/// that works while the player sits in the ship and the route is idle, and says to take off
/// while the ship stands on the ground.
#[test]
fn the_offer_without_a_place_and_for_a_ready_ship() {
    let keys = keys();
    let nothing = offer(None);
    assert_eq!(
        (nothing.button, nothing.sentence.as_str()),
        (None, NO_PLACE)
    );
    let go = Button {
        stop: false,
        enabled: true,
        home: false,
    };
    let ready = offer(Some(&base(&keys)));
    assert_eq!(
        (ready.button, ready.sentence.as_str()),
        (Some(go), "About 4 min.")
    );
    let landed = Ask {
        status: RouteStatus::Blocked(Block::Landed),
        ..base(&keys)
    };
    assert_eq!(offer(Some(&landed)).sentence, "Take off first: hold Space.");
}

/// The button reads Stop while the route to the place is flown, and is dim, with its reason,
/// while the player is on foot or the ship stands on the ground.
#[test]
fn the_offer_stops_a_flown_route_and_explains_a_blocked_one() {
    let keys = keys();
    let flown = Ask {
        ours: true,
        status: RouteStatus::InWarp,
        ..base(&keys)
    };
    let stop = Button {
        stop: true,
        enabled: true,
        home: false,
    };
    let warp = offer(Some(&flown));
    assert_eq!(warp.button, Some(stop));
    assert_eq!(warp.sentence, "Warping to Mars. G stops.");
    let on_foot = Ask {
        seated: false,
        ..base(&keys)
    };
    let dim = Button {
        stop: false,
        enabled: false,
        home: false,
    };
    let foot = offer(Some(&on_foot));
    assert_eq!(foot.button, Some(dim));
    assert!(foot.sentence.starts_with("Board your ship first"));
}

/// Returns an app with a table of one planet, the player seated, the route idle, and the card.
fn rig() -> (App, Target) {
    let universe = Universe::new(7);
    let planet = Target::Body(universe.home());
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Board>()
        .init_resource::<MapView>()
        .init_resource::<Destination>()
        .init_resource::<RouteStatus>()
        .insert_resource(Location {
            body: universe.home(),
            travel: Travel::InShip,
        })
        .insert_resource(universe);
    app.world_mut().spawn(deeds());
    (app, planet)
}

/// Returns the row of the planet `target` in a table, with no distance to speak of.
fn row(target: Target) -> Line {
    Line {
        title: "Kolva".to_string(),
        detail: String::new(),
        target,
        distance: 0.0,
        visited: true,
        at: default(),
    }
}

/// Writes the card and returns its sentence and the acts of its buttons.
fn shown(app: &mut App) -> (String, Vec<Act>) {
    app.world_mut().run_system_once(write).unwrap();
    let world = app.world_mut();
    let acts = world.query::<&Act>().iter(world).copied().collect();
    let said = world
        .query_filtered::<&Text, With<Sentence>>()
        .single(world)
        .unwrap();
    (said.0.clone(), acts)
}

/// With no row picked the card says how to pick one. Picking a row far from the player gives it
/// the Go button, and the card of the player's own place says so and has none that works.
#[test]
fn the_card_follows_the_picked_row() {
    let (mut app, planet) = rig();
    assert_eq!(shown(&mut app), (NO_PLACE.to_string(), vec![]));
    let mut board = app.world_mut().resource_mut::<Board>();
    board.fill("Varena system".to_string(), vec![row(planet)]);
    assert_eq!(shown(&mut app), (NO_PLACE.to_string(), vec![]));
    app.world_mut().resource_mut::<MapView>().picked = true;
    let (said, acts) = shown(&mut app);
    assert_eq!(acts, vec![Act::Go]);
    assert_eq!(said, "The autopilot flies there.");
    app.world_mut().resource_mut::<Board>().here[0] = Some(planet);
    let (said, acts) = shown(&mut app);
    assert_eq!((said.as_str(), acts), ("You are here.", vec![]));
    let mut rows = app.world_mut().query_filtered::<Entity, With<Deeds>>();
    assert_eq!(rows.iter(app.world()).count(), 1);
}
