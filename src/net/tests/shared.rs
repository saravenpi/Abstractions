//! What a joined world shares with the server: the log and the atlas, and what it never
//! writes to the local saves folder.

use abstractions_net::io::ClientEvent;
use abstractions_net::log::{Change, Changes, Entry, PlayerId};
use abstractions_net::wire::{self, Body, ClientMsg, Head, Kind, Record, ServerMsg, ShipBody};
use bevy::math::DVec3;
use bevy::prelude::*;

use super::join::{moon, visited};
use super::rig::{app, push, run, scripted, sent, welcome};
use crate::computer::{Atlas, Target};
use crate::menus::{MarkerKind, MarkerPlace, Markers, Menu};
use crate::saves::{SaveDir, Writer};
use crate::screens::Screen;
use crate::universe::{BodyId, StarId, Universe};

/// Returns an app joined to a scripted world of seed 11 whose log holds a visit of its first
/// moon, and the script.
fn joined(tag: &str) -> (App, crate::net::link::Script) {
    let mut app = app(tag);
    let script = scripted(&mut app);
    let hello = welcome(11, visited(moon(11, 1)), None);
    push(&script, ClientEvent::Message(ServerMsg::Welcome(hello)));
    run(&mut app, 3);
    (app, script)
}

/// Returns the bodies the log holds as visited.
fn logged(app: &App) -> Vec<Change> {
    let changes = app.world().resource::<Changes>();
    changes
        .entries
        .iter()
        .map(|entry| entry.change.clone())
        .collect()
}

/// A change another player made joins the log and the atlas, and is not sent back.
#[test]
fn a_change_of_the_server_joins_the_log_and_the_atlas() {
    let (mut app, script) = joined("change");
    let found = moon(11, 2);
    let entry = Entry {
        player: PlayerId(1),
        time: 50.0,
        change: Change::Visited(found),
    };
    push(&script, ClientEvent::Message(ServerMsg::Change(entry)));
    run(&mut app, 3);
    let both = [Change::Visited(moon(11, 1)), Change::Visited(found)];
    assert_eq!(logged(&app), both);
    assert!(app.world().resource::<Atlas>().bodies.contains(&found));
    let resent = sent(&script)
        .into_iter()
        .any(|msg| matches!(msg, ClientMsg::Change(_)));
    assert!(!resent, "a change of the server is never sent back");
}

/// A first visit of the local player goes to the server instead of the local log.
#[test]
fn a_local_visit_is_sent_instead_of_logged() {
    let (mut app, script) = joined("visit");
    let found = moon(11, 3);
    app.world_mut()
        .resource_mut::<Atlas>()
        .visit(Target::Body(found));
    run(&mut app, 3);
    assert_eq!(logged(&app), [Change::Visited(moon(11, 1))]);
    let changes: Vec<ClientMsg> = sent(&script)
        .into_iter()
        .filter(|msg| matches!(msg, ClientMsg::Change(_)))
        .collect();
    assert_eq!(changes, [ClientMsg::Change(Change::Visited(found))]);
}

/// Pausing and leaving a joined world save nothing to the local saves folder.
#[test]
fn a_joined_world_is_never_written() {
    let (mut app, _script) = joined("unsaved");
    let world = app.world_mut();
    world
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Gameplay);
    run(&mut app, 2);
    let world = app.world_mut();
    world.resource_mut::<NextState<Menu>>().set(Menu::Pause);
    run(&mut app, 2);
    let world = app.world_mut();
    world.resource_mut::<NextState<Screen>>().set(Screen::Title);
    run(&mut app, 3);
    assert!(!app.world().resource::<Writer>().busy());
    let dir = &app.world().resource::<SaveDir>().0;
    let files = std::fs::read_dir(dir).map_or(0, |entries| entries.count());
    assert_eq!(files, 0, "nothing in {}", dir.display());
}

/// Returns a star inside a galaxy different from home in seed `seed`.
fn other_star(seed: u64) -> StarId {
    let universe = Universe::new(seed);
    let other = Atlas::new(&universe).known_galaxies(&universe)[1].clone();
    universe.stars_near(other.id, DVec3::X * other.radius * 0.05, 20.0)[0].id
}

/// A visited body change in another galaxy reveals its star and galaxy alongside the body.
#[test]
fn a_remote_visit_in_another_galaxy_reveals_its_galaxy() {
    let (mut app, script) = joined("far_visit");
    let star = other_star(11);
    let far = BodyId {
        star,
        index: 0,
        moon: 0,
    };
    let entry = Entry {
        player: PlayerId(1),
        time: 50.0,
        change: Change::Visited(far),
    };
    push(&script, ClientEvent::Message(ServerMsg::Change(entry)));
    run(&mut app, 3);
    let atlas = app.world().resource::<Atlas>();
    assert!(atlas.bodies.contains(&far));
    assert!(atlas.visited(Target::Star(star)));
    assert!(atlas.visited(Target::Galaxy(star.galaxy)));
}

/// A friend in another galaxy reveals their galaxy/star and keeps a `Star` marker cross-galaxy.
#[test]
fn cross_galaxy_friend_reveals_galaxy_and_keeps_star_marker() {
    let (mut app, script) = joined("far_friend");
    app.insert_resource(Universe::new(11))
        .init_resource::<Markers>();
    let star = other_star(11);
    let frame = wire::Frame::System(star);
    let mut bytes = Vec::new();
    ShipBody::default().encode(&frame, &mut bytes);
    let mut batch = Vec::new();
    let head = Head {
        player: PlayerId(1),
        kind: Kind::Ship,
        frame,
        time: 1234.5,
        flags: 0,
    };
    wire::push_within(&mut batch, &Record { head, body: &bytes }, 1200);
    push(&script, ClientEvent::Datagram(batch));
    run(&mut app, 3);
    let atlas = app.world().resource::<Atlas>();
    assert!(atlas.visited(Target::Galaxy(star.galaxy)) && atlas.visited(Target::Star(star)));
    let markers = &app.world().resource::<Markers>().0;
    assert!(
        markers.iter().any(
            |m| m.kind == MarkerKind::Friend && m.place == MarkerPlace::Star(star, DVec3::ZERO)
        )
    );
}
