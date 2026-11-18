//! The multiplayer menu inside the app: the first name, adding, editing and removing servers,
//! and joining until the session ends.

use bevy::input::keyboard::Key;
use bevy::prelude::*;

use super::flow::menu;
use super::kept::{kept, watch};
use super::{activate, app, press, stored, tap};
use crate::menus::multiplayer::Act;
use crate::menus::{Menu, Nav};
use crate::net::{JoinServer, LeaveSession, RefreshLobby, Session, SessionEnded};
use crate::saves::SaveDir;
use crate::settings::{Persistent, SavedServer, Settings};

/// Returns the settings as the file on disk holds them.
fn on_disk(app: &App) -> Settings {
    let worlds = &app.world().resource::<SaveDir>().0;
    stored(worlds.parent().unwrap()).get().clone()
}

/// Types `text`, one key per character, then runs a frame.
fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        let typed = c.to_string();
        let logical = Key::Character(typed.as_str().into());
        press(app, KeyCode::KeyA, logical, Some(&typed));
    }
}

/// Returns an app on the multiplayer menu, opened from the main menu, with `name` as the player
/// name and the requests to the network kept.
fn on_multiplayer(tag: &str, name: &str) -> App {
    let mut app = app(tag);
    watch::<RefreshLobby>(&mut app);
    watch::<JoinServer>(&mut app);
    watch::<LeaveSession>(&mut app);
    app.world_mut()
        .resource_mut::<Persistent<Settings>>()
        .player_name = name.to_string();
    activate(&mut app, Nav::To(Menu::Multiplayer));
    assert_eq!(menu(&app), Menu::Multiplayer);
    app
}

/// Without a name the menu asks for one first: Escape leaves, and an accepted name is saved.
#[test]
fn a_first_visit_asks_for_the_player_name() {
    let mut app = on_multiplayer("mp-name", "");
    assert_eq!(kept::<RefreshLobby>(&mut app).len(), 1);
    tap(&mut app, KeyCode::Escape);
    app.update();
    assert_eq!(menu(&app), Menu::Main);
    activate(&mut app, Nav::To(Menu::Multiplayer));
    tap(&mut app, KeyCode::Enter);
    assert_eq!(on_disk(&app).player_name, "", "an empty name is refused");
    type_text(&mut app, " Ada ");
    tap(&mut app, KeyCode::Enter);
    assert_eq!(on_disk(&app).player_name, "Ada");
    assert_eq!(menu(&app), Menu::Multiplayer);
}

/// A adds a server, E edits the selected one and Delete removes it; each change is on disk.
#[test]
fn servers_are_added_edited_and_removed() {
    let mut app = on_multiplayer("mp-servers", "Ada");
    let space = on_disk(&app).servers;
    tap(&mut app, KeyCode::KeyA);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(on_disk(&app).servers, space, "a server needs an address");
    type_text(&mut app, "Home");
    press(&mut app, KeyCode::Tab, Key::Tab, None);
    type_text(&mut app, "10.0.0.2");
    tap(&mut app, KeyCode::Enter);
    let home = SavedServer {
        name: "Home".to_string(),
        address: "10.0.0.2".to_string(),
    };
    assert_eq!(on_disk(&app).servers, [space[0].clone(), home.clone()]);
    tap(&mut app, KeyCode::KeyE);
    type_text(&mut app, "s");
    tap(&mut app, KeyCode::Enter);
    assert_eq!(on_disk(&app).servers[1].name, "Homes");
    tap(&mut app, KeyCode::Delete);
    assert_eq!(on_disk(&app).servers, space);
    tap(&mut app, KeyCode::Delete);
    assert!(
        on_disk(&app).servers.is_empty(),
        "the public server is removable"
    );
    assert_eq!(menu(&app), Menu::Multiplayer);
}

/// Sets the session and runs a few frames.
fn set_session(app: &mut App, session: Session) {
    app.insert_resource(session);
    for _ in 0..3 {
        app.update();
    }
}

/// Join asks the network and waits for it; Joining opens the joining screen, and Cancel leaves
/// without opening it again.
#[test]
fn joining_opens_the_joining_screen() {
    let mut app = on_multiplayer("mp-join", "Ada");
    let joining = Session::Joining {
        address: "space.facile.studio".to_string(),
    };
    set_session(&mut app, joining);
    assert_eq!(
        menu(&app),
        Menu::Multiplayer,
        "only a join of this menu opens it"
    );
    activate(&mut app, Act::Join);
    let asked = kept::<JoinServer>(&mut app);
    assert_eq!(asked[0].address, "space.facile.studio");
    assert_eq!(asked[0].expected_key, None);
    app.update();
    assert_eq!(menu(&app), Menu::Joining);
    activate(&mut app, Nav::Back);
    app.update();
    assert_eq!(menu(&app), Menu::Multiplayer);
    assert_eq!(kept::<LeaveSession>(&mut app).len(), 1);
}

/// A session that ends while joining comes back to the menu with its reason.
#[test]
fn an_ended_session_shows_why() {
    let mut app = on_multiplayer("mp-ended", "Ada");
    activate(&mut app, Act::Join);
    let address = "space.facile.studio".to_string();
    set_session(&mut app, Session::Joining { address });
    assert_eq!(menu(&app), Menu::Joining);
    app.world_mut().write_message(SessionEnded {
        reason: "The server is full".to_string(),
    });
    set_session(&mut app, Session::Offline);
    assert_eq!(menu(&app), Menu::Multiplayer);
    let mut texts = app.world_mut().query::<&Text>();
    let shown = texts
        .iter(app.world())
        .any(|text| text.0 == "The server is full");
    assert!(shown, "the reason shows on the menu");
    tap(&mut app, KeyCode::Escape);
    assert_eq!(menu(&app), Menu::Main);
}
