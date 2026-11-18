//! Flow tests: every menu opens and closes, Escape goes back one level, the last entry is
//! remembered and the interface sounds are sent once.

use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use super::heard::cues;
use super::{activate, app, tap};
use crate::audio::AudioCue;
use crate::menus::recall::Entry;
use crate::menus::{Menu, Nav, Trail};
use crate::ui::{Cancels, Overlay};

/// Returns the open menu.
pub(super) fn menu(app: &App) -> Menu {
    *app.world().resource::<State<Menu>>().get()
}

/// Opens `wanted` directly and runs a few frames.
pub(super) fn open(app: &mut App, wanted: Menu) {
    app.world_mut()
        .resource_mut::<NextState<Menu>>()
        .set(wanted);
    for _ in 0..3 {
        app.update();
    }
}

/// Returns the number of full-screen roots alive.
fn roots(app: &mut App) -> usize {
    let mut roots = app.world_mut().query_filtered::<(), With<Overlay>>();
    roots.iter(app.world()).count()
}

/// The menus that can be opened.
const MENUS: [Menu; 8] = [
    Menu::Main,
    Menu::Worlds,
    Menu::NewGame,
    Menu::Settings,
    Menu::Pause,
    Menu::Multiplayer,
    Menu::Joining,
    Menu::Players,
];

/// Every menu opens with one focused button, runs, and leaves nothing behind when it closes.
#[test]
fn every_menu_opens_and_closes() {
    let mut app = app("open-close");
    assert_eq!(menu(&app), Menu::Main);
    open(&mut app, Menu::None);
    let title_only = roots(&mut app);
    for wanted in MENUS {
        open(&mut app, wanted);
        assert_eq!(menu(&app), wanted);
        assert_eq!(roots(&mut app), title_only + 1, "{wanted:?}");
        let focus = app.world().resource::<InputFocus>().get();
        assert!(focus.is_some(), "{wanted:?}");
        open(&mut app, Menu::None);
        assert_eq!(roots(&mut app), title_only, "{wanted:?}");
    }
}

/// Escape goes back one level from every depth, and the main menu is the top.
#[test]
fn escape_goes_back_one_level() {
    let mut app = app("escape");
    activate(&mut app, Nav::To(Menu::Worlds));
    assert_eq!(menu(&app), Menu::Worlds);
    tap(&mut app, KeyCode::KeyN);
    app.update();
    assert_eq!(menu(&app), Menu::NewGame);
    assert_eq!(
        app.world().resource::<Trail>().0,
        [Menu::Main, Menu::Worlds]
    );
    tap(&mut app, KeyCode::Escape);
    assert_eq!(menu(&app), Menu::Worlds);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(menu(&app), Menu::Main);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(menu(&app), Menu::Main);
    activate(&mut app, Nav::To(Menu::Settings));
    tap(&mut app, KeyCode::Escape);
    assert_eq!(menu(&app), Menu::Main);
    open(&mut app, Menu::Pause);
    activate(&mut app, Nav::To(Menu::Settings));
    assert_eq!(menu(&app), Menu::Settings);
    activate(&mut app, Nav::Back);
    assert_eq!(menu(&app), Menu::Pause);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(menu(&app), Menu::None);
}

/// The entry used last takes the focus when its menu opens again.
#[test]
fn the_last_entry_is_remembered() {
    let mut app = app("recall");
    activate(&mut app, Nav::To(Menu::Settings));
    tap(&mut app, KeyCode::Escape);
    app.update();
    assert_eq!(menu(&app), Menu::Main);
    let focused = app.world().resource::<InputFocus>().get().unwrap();
    assert_eq!(
        app.world().get::<Nav>(focused),
        Some(&Nav::To(Menu::Settings))
    );
    assert_eq!(app.world().get::<Entry>(focused), Some(&Entry(4)));
}

/// A button sounds one confirm, a back button and Escape one cancel, a focus move one move.
#[test]
fn interface_sounds_are_sent_once() {
    let mut app = app("sounds");
    cues(&mut app);
    activate(&mut app, Nav::To(Menu::Settings));
    assert_eq!(cues(&mut app), [AudioCue::UiConfirm]);
    let mut backs = app.world_mut().query_filtered::<Entity, With<Cancels>>();
    let back = backs.single(app.world()).unwrap();
    let mut focus = app.world_mut().resource_mut::<InputFocus>();
    focus.set(back, FocusCause::Navigated);
    app.update();
    assert_eq!(cues(&mut app), [AudioCue::UiMove]);
    activate(&mut app, Nav::Back);
    assert_eq!(cues(&mut app), [AudioCue::UiCancel]);
    activate(&mut app, Nav::To(Menu::NewGame));
    cues(&mut app);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(cues(&mut app), [AudioCue::UiCancel]);
}
