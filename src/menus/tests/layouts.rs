//! Keyboard layouts inside the app: a rebind keeps the physical key whatever the layout prints
//! on it, and what is typed in a text field never runs a shortcut.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::flow::{menu, open};
use super::worlds::{on_disk, with_two_worlds};
use super::{activate, app, press, tap};
use crate::menus::Menu;
use crate::menus::options::{Bind, SettingsAction, SettingsTab, Tab};
use crate::settings::{KeyLabels, Persistent, Settings, key_labels};

/// The keys of the shortcuts of the worlds menu, with the letters they type: copy, sort and
/// new world.
const SHORTCUTS: [(KeyCode, &str); 3] = [
    (KeyCode::KeyC, "c"),
    (KeyCode::KeyT, "t"),
    (KeyCode::KeyN, "n"),
];

/// Returns every text on screen.
fn texts(app: &mut App) -> Vec<String> {
    let mut texts = app.world_mut().query::<&Text>();
    texts.iter(app.world()).map(|text| text.0.clone()).collect()
}

/// Returns the key bound to walking forward.
fn forward(app: &App) -> KeyCode {
    let settings = app.world().resource::<Persistent<Settings>>();
    settings.controls.keys.forward
}

/// Presses or releases AltGr, which types nothing.
fn alt_gr(app: &mut App, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::AltRight,
        logical_key: Key::AltGraph,
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

/// On QWERTZ the key where Y is on QWERTY types Z. Rebinding to it stores that physical key
/// and the row shows Z. When the layout then changes to QWERTY, the binding stays on the same
/// key and its label follows the layout.
#[test]
fn a_rebind_keeps_the_physical_key_when_the_layout_changes() {
    let mut app = app("layout-rebind");
    app.add_plugins(key_labels);
    open(&mut app, Menu::Settings);
    app.world_mut().resource_mut::<SettingsTab>().0 = Tab::Controls;
    app.update();
    activate(&mut app, SettingsAction::Rebind(Bind::Forward));
    assert!(texts(&mut app).contains(&"Press a key".to_string()));
    press(
        &mut app,
        KeyCode::KeyY,
        Key::Character("z".into()),
        Some("z"),
    );
    app.update();
    assert_eq!(forward(&app), KeyCode::KeyY, "the physical key is stored");
    assert!(
        texts(&mut app).contains(&"Z".to_string()),
        "as QWERTZ prints"
    );
    press(
        &mut app,
        KeyCode::KeyY,
        Key::Character("y".into()),
        Some("y"),
    );
    app.update();
    assert_eq!(forward(&app), KeyCode::KeyY, "the binding did not move");
    let labels = *app.world().resource::<KeyLabels>();
    assert_eq!(labels.label(forward(&app)), "Y", "as QWERTY prints");
    assert!(!texts(&mut app).contains(&"Press a key".to_string()));
}

/// The row that resets the keys puts a rebound key back on its default.
#[test]
fn the_reset_row_restores_the_default_keys() {
    let mut app = app("layout-reset");
    open(&mut app, Menu::Settings);
    activate(&mut app, SettingsAction::Rebind(Bind::Forward));
    press(
        &mut app,
        KeyCode::KeyP,
        Key::Character("p".into()),
        Some("p"),
    );
    assert_eq!(forward(&app), KeyCode::KeyP);
    let mut rows = app.world_mut().query::<(Entity, &SettingsAction)>();
    let reset = rows
        .iter(app.world())
        .find(|(_, action)| **action == SettingsAction::ResetKeys);
    let (entity, _) = reset.expect("a row resets the keys");
    app.world_mut()
        .trigger(bevy::ui_widgets::Activate { entity });
    app.update();
    assert_eq!(forward(&app), KeyCode::KeyW);
}

/// While a world is renamed, an accent, a sign typed with AltGr, a letter made with a dead key
/// and the letters of the shortcuts of the menu all go into the name: no world is copied, the
/// order does not change and the new world menu does not open.
#[test]
fn typing_in_a_field_runs_no_shortcut() {
    let mut app = with_two_worlds("layout-typing");
    let letter = |app: &mut App, code, typed: &str, text: &str| {
        press(app, code, Key::Character(typed.into()), Some(text));
    };
    tap(&mut app, KeyCode::KeyR);
    letter(&mut app, KeyCode::Digit2, "é", "é");
    alt_gr(&mut app, ButtonState::Pressed);
    letter(&mut app, KeyCode::Digit0, "@", "@");
    alt_gr(&mut app, ButtonState::Released);
    for (code, typed) in SHORTCUTS {
        letter(&mut app, code, typed, typed);
    }
    press(&mut app, KeyCode::BracketLeft, Key::Dead(Some('^')), None);
    letter(&mut app, KeyCode::KeyE, "e", "ê");
    assert_eq!(menu(&app), Menu::Worlds, "N typed a letter");
    tap(&mut app, KeyCode::Enter);
    let worlds = on_disk(&app);
    assert_eq!(worlds.len(), 2, "C typed a letter");
    assert_eq!(worlds[1].save.name, "Reefé@ctnê");
}
