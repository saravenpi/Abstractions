//! The password prompt: after a server refused the password, the multiplayer menu asks for
//! one, hides it as it is typed, and joins the same server again with it.

use bevy::input::keyboard::Key;
use bevy::prelude::*;

use super::flow::menu;
use super::kept::{kept, watch};
use super::{activate, app, press, tap};
use crate::menus::{Menu, Nav};
use crate::net::{JoinServer, PasswordNeeded};
use crate::settings::{Persistent, Settings};

/// Returns every text on screen.
fn texts(app: &mut App) -> Vec<String> {
    let mut texts = app.world_mut().query::<&Text>();
    texts.iter(app.world()).map(|text| text.0.clone()).collect()
}

/// A refused password opens the prompt on the menu; the typed password shows as stars and
/// Enter joins the same server with it and the key it had to show.
#[test]
fn a_refused_password_is_asked_and_sent_again() {
    let mut app = app("mp-password");
    watch::<JoinServer>(&mut app);
    app.world_mut()
        .resource_mut::<Persistent<Settings>>()
        .player_name = "Ada".to_string();
    app.world_mut().write_message(PasswordNeeded {
        address: "home.example".to_string(),
        expected_key: Some([3; 32]),
    });
    activate(&mut app, Nav::To(Menu::Multiplayer));
    app.update();
    for letter in ["a", "b", "c"] {
        let logical = Key::Character(letter.into());
        press(&mut app, KeyCode::KeyA, logical, Some(letter));
    }
    let shown = texts(&mut app);
    assert!(
        shown.iter().any(|text| text.starts_with("***")),
        "{shown:?}"
    );
    assert!(!shown.iter().any(|text| text.contains("abc")));
    tap(&mut app, KeyCode::Enter);
    let asked = kept::<JoinServer>(&mut app);
    let again = JoinServer {
        address: "home.example".to_string(),
        expected_key: Some([3; 32]),
        password: Some("abc".to_string()),
    };
    assert_eq!(asked, [again]);
    assert_eq!(menu(&app), Menu::Multiplayer);
}
