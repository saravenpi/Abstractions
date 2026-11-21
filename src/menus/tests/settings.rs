//! The settings menu inside the app: every page shows, the audio sliders step the volumes and a
//! key cut short is named in full in its tooltip.

use bevy::prelude::*;

use super::flow::open;
use super::{activate, app};
use crate::menus::Menu;
use crate::menus::options::{Field, SettingsAction, SettingsTab, Tab};
use crate::settings::{Persistent, Settings};
use crate::ui::TipText;

/// Every page of the settings shows without a panic.
#[test]
fn every_settings_page_shows() {
    let mut app = app("pages");
    open(&mut app, Menu::Settings);
    for tab in Tab::ALL {
        app.world_mut().resource_mut::<SettingsTab>().0 = tab;
        app.update();
        app.update();
    }
    assert_eq!(app.world().resource::<SettingsTab>().0, Tab::Help);
}

/// The audio sliders still step the volumes, which the audio module reads from the settings.
#[test]
fn audio_sliders_change_the_volumes() {
    let mut app = app("audio");
    open(&mut app, Menu::Settings);
    activate(&mut app, SettingsAction::Adjust(Field::Music, -1));
    activate(&mut app, SettingsAction::Adjust(Field::Master, -1));
    activate(&mut app, SettingsAction::Adjust(Field::Effects, -1));
    let audio = app.world().resource::<Persistent<Settings>>().audio.clone();
    assert_eq!((audio.master, audio.music, audio.effects), (0.9, 0.7, 0.7));
}

/// A key whose name the row cuts short is named in full in the tooltip of its row, and only
/// that row has a tooltip.
#[test]
fn a_key_cut_short_is_named_in_full_in_its_tooltip() {
    let mut app = app("key-tip");
    let mut settings = app.world_mut().resource_mut::<Persistent<Settings>>();
    let long = |s: &mut Settings| s.controls.keys.jump = KeyCode::NumpadMemorySubtract;
    settings.update(long).expect("the settings save");
    open(&mut app, Menu::Settings);
    let mut rows = app.world_mut().query::<(&SettingsAction, &TipText)>();
    let said: Vec<String> = rows
        .iter(app.world())
        .filter(|(action, tip)| matches!(action, SettingsAction::Rebind(_)) && !tip.0.is_empty())
        .map(|(_, tip)| tip.0.clone())
        .collect();
    assert_eq!(said, ["Num MemorySubtract"]);
}
