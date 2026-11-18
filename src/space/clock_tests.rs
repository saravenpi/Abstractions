//! Clock tests: universe time stops behind a menu in single player and runs on in a session.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::Clock;
use crate::menus::Menu;
use crate::net::Session;
use crate::screens::Screen;

/// Returns the universe seconds after ten frames of gameplay with `menu` open in `session`.
fn seconds_with(menu: Menu, session: Session) -> f64 {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )))
        .init_state::<Screen>()
        .init_state::<Menu>()
        .insert_resource(session);
    super::clock::plugin(&mut app);
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Gameplay);
    app.world_mut().resource_mut::<NextState<Menu>>().set(menu);
    for _ in 0..10 {
        app.update();
    }
    app.world().resource::<Clock>().seconds
}

/// The pause menu stops the clock alone; joined or hosting, the clock runs behind any menu.
#[test]
fn the_clock_runs_behind_menus_in_a_session() {
    let joined = Session::Joined {
        address: "space.facile.studio".to_string(),
        name: "Space".to_string(),
    };
    let hosting = Session::Hosting {
        lan_address: "192.168.1.20:24650".to_string(),
        key: "7F3A-91C2-0B5E".to_string(),
    };
    assert_eq!(seconds_with(Menu::Pause, Session::Offline), 0.0);
    assert!(seconds_with(Menu::None, Session::Offline) > 0.0);
    assert!(seconds_with(Menu::Pause, joined) > 0.0);
    assert!(seconds_with(Menu::Settings, hosting) > 0.0);
}
