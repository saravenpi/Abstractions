//! Fit tests of every full-screen screen of the menus, the title screen behind them too: each is
//! the size of the window, and what it shows fits the narrowest and the shortest layout any
//! window gives at any interface size, its lists scrolling in the room left.

use bevy::prelude::*;

use super::app;
use super::flow::open;
use super::rig::{content, overflows};
use super::stage::{narrowest, shortest};
use super::tall_rig::inside;
use crate::menus::Menu;
use crate::menus::options::{SettingsTab, Tab};
use crate::ui::Overlay;

/// Every menu that can be opened.
const SCREENS: [Menu; 8] = [
    Menu::Main,
    Menu::Worlds,
    Menu::NewGame,
    Menu::Settings,
    Menu::Pause,
    Menu::Multiplayer,
    Menu::Joining,
    Menu::Players,
];

/// Returns the pixels a padding set as a share of the window takes in a window `room` wide.
fn share_of(value: Val, room: f32) -> f32 {
    match value {
        Val::Percent(part) => part / 100.0 * room,
        _ => 0.0,
    }
}

/// Returns `root` and every node under it, `root` first.
pub(super) fn under(world: &World, root: Entity) -> Vec<Entity> {
    let mut found = vec![root];
    let mut at = 0;
    while at < found.len() {
        if let Some(children) = world.get::<Children>(found[at]) {
            found.extend(children.into_iter().copied());
        }
        at += 1;
    }
    found
}

/// Returns a line for each way a full-screen root of the world does not fit a layout `room`
/// large: it is not the window, a box does not hold its content, it is too wide or too tall.
fn misfits(world: &mut World, room: Vec2) -> Vec<String> {
    let mut roots = world.query_filtered::<Entity, (With<Overlay>, Without<ChildOf>)>();
    let found: Vec<Entity> = roots.iter(world).collect();
    let world = &*world;
    let mut said = Vec::new();
    for root in found {
        let node = world.get::<Node>(root).expect("a root node");
        if (node.width, node.height) != (vw(100), vh(100)) {
            said.push(format!("{root} is {:?} by {:?}", node.width, node.height));
        }
        said.extend(overflows(world, &under(world, root)));
        let sides = share_of(node.padding.left, room.x) + share_of(node.padding.right, room.x);
        let across = content(world, root) + sides;
        let tall = inside(world, root, room.x);
        if across > room.x || tall > room.y {
            said.push(format!("{root} needs {across} by {tall} in {room}"));
        }
    }
    said
}

/// Every screen, every tab of the settings and the title screen behind them is as large as the
/// window, and fits the narrowest and the shortest layout of every window at every interface
/// size, with its lists scrolling in what is left.
#[test]
fn every_screen_is_the_window_and_fits_it() {
    let mut app = app("screens-fit");
    let room = Vec2::new(narrowest(), shortest());
    for menu in SCREENS {
        open(&mut app, menu);
        let said = misfits(app.world_mut(), room);
        assert!(said.is_empty(), "{menu:?}: {said:#?}");
    }
    open(&mut app, Menu::Settings);
    for tab in Tab::ALL {
        app.world_mut().resource_mut::<SettingsTab>().0 = tab;
        app.update();
        let said = misfits(app.world_mut(), room);
        assert!(said.is_empty(), "{tab:?}: {said:#?}");
    }
}
