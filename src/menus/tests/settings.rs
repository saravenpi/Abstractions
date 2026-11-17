//! The settings menu inside the app: every page shows and the audio sliders step the volumes.

use super::flow::open;
use super::{activate, app};
use crate::menus::Menu;
use crate::menus::options::{Field, SettingsAction, SettingsTab, Tab};
use crate::settings::{Persistent, Settings};

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
