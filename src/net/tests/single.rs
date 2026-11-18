//! Single player runs no network code: no link, no listener, no server, no request.

use crate::menus::Menu;
use crate::net::host::Hosted;
use crate::net::link::Link;
use crate::net::scan::{Scan, flying};
use crate::net::{Inbox, Session};
use crate::screens::Screen;

use super::rig::{app, run};
use bevy::prelude::*;

/// A world played alone, paused and resumed, never makes a link, a listener or a server,
/// and the session never moves from offline.
#[test]
fn single_player_has_no_network() {
    let mut app = app("single");
    let world = app.world_mut();
    world
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Gameplay);
    run(&mut app, 3);
    for menu in [Menu::Pause, Menu::None, Menu::Pause, Menu::None] {
        app.world_mut().resource_mut::<NextState<Menu>>().set(menu);
        run(&mut app, 2);
    }
    assert!(!app.world_mut().run_system_cached(flying).unwrap());
    let world = app.world();
    assert!(!world.contains_resource::<Link>());
    assert!(!world.contains_resource::<Inbox>());
    assert!(!world.contains_resource::<Scan>());
    assert!(!world.contains_resource::<Hosted>());
    assert_eq!(*world.resource::<Session>(), Session::Offline);
}
