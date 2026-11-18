//! The pause menu in each session: its entries, the corner, the requests it sends, the players
//! menu, and physics that freezes only in single player.

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;

use super::flow::open;
use super::kept::{kept, watch};
use super::{activate, app};
use crate::menus::pause::{Online, entries};
use crate::menus::{Menu, Nav};
use crate::net::{OpenToLan, Players, Session};

/// Returns every text on screen.
fn texts(app: &mut App) -> Vec<String> {
    let mut texts = app.world_mut().query::<&Text>();
    texts.iter(app.world()).map(|text| text.0.clone()).collect()
}

/// Returns whether physics is frozen.
fn frozen(app: &App) -> bool {
    app.world().resource::<Time<Physics>>().is_paused()
}

/// Returns the labels of the pause entries for `session`.
fn labels(session: &Session) -> Vec<&'static str> {
    entries(session)
        .into_iter()
        .map(|(_, label, _)| label)
        .collect()
}

/// Returns a session hosting on the local network.
fn hosting() -> Session {
    Session::Hosting {
        lan_address: "192.168.1.20:24650".to_string(),
        key: "7F3A-91C2-0B5E".to_string(),
    }
}

/// Returns a session joined to the public server.
fn joined() -> Session {
    Session::Joined {
        address: "space.facile.studio".to_string(),
        name: "Space".to_string(),
    }
}

/// Single player and hosting save on quit and open or close the LAN; joined, the server saves,
/// so the entries leave it and show the players.
#[test]
fn the_entries_follow_the_session() {
    let single = ["Resume", "Settings", "Open to LAN"];
    assert_eq!(labels(&Session::Offline)[..3], single);
    assert_eq!(labels(&hosting())[2], "Close LAN");
    assert_eq!(labels(&hosting())[3], "Save and quit to menu");
    let online = [
        "Resume",
        "Players",
        "Settings",
        "Disconnect",
        "Quit the game",
    ];
    assert_eq!(labels(&joined()), online);
}

/// Offline the pause menu freezes physics and opens to LAN; hosting, it says where and with
/// which key, and physics runs on; joined, it lists the players.
#[test]
fn the_pause_menu_in_each_session() {
    let mut app = app("pause-session");
    watch::<OpenToLan>(&mut app);
    open(&mut app, Menu::Pause);
    assert!(frozen(&app));
    activate(&mut app, Online::OpenToLan);
    assert_eq!(kept::<OpenToLan>(&mut app).len(), 1);
    app.insert_resource(hosting());
    app.update();
    app.update();
    assert!(!frozen(&app), "a session never freezes");
    let shown = texts(&mut app);
    assert!(shown.contains(&"Close LAN".to_string()));
    assert!(shown.contains(&"Hosting on 192.168.1.20:24650 - key 7F3A-91C2-0B5E".to_string()));
    open(&mut app, Menu::None);
    app.insert_resource(joined());
    app.insert_resource(Players(vec![
        (1, "Ada".to_string()),
        (2, "Lin".to_string()),
    ]));
    open(&mut app, Menu::Pause);
    assert!(!frozen(&app));
    activate(&mut app, Nav::To(Menu::Players));
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Players);
    let shown = texts(&mut app);
    assert!(shown.contains(&"Ada".to_string()) && shown.contains(&"Lin".to_string()));
}
