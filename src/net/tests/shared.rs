//! What a joined world shares with the server: the log and the atlas, and what it never
//! writes to the local saves folder.

use abstractions_net::io::ClientEvent;
use abstractions_net::log::{Change, Changes, Entry, PlayerId};
use abstractions_net::wire::{ClientMsg, ServerMsg};
use bevy::prelude::*;

use super::join::{moon, visited};
use super::rig::{app, push, run, scripted, sent, welcome};
use crate::computer::{Atlas, Target};
use crate::menus::Menu;
use crate::saves::{SaveDir, Writer};
use crate::screens::Screen;

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
