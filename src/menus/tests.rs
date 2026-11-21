//! Headless app tests: the menus, the HUD, the boot screen and the board computer inside an app
//! without a window. This file is the harness; the tests are in the modules below.

mod computer;
mod fit;
mod fit_screens;
mod flow;
mod heard;
mod kept;
mod look;
mod multiplayer;
mod password;
mod rig;
mod rules;
mod rules_colour;
mod rules_page;
mod schedules;
mod session;
mod settings;
pub(super) mod stage;
mod tall_rig;
mod update;
mod worlds;

use std::path::PathBuf;
use std::time::Duration;

use avian3d::prelude::Physics;
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::{ButtonState, InputPlugin};
use bevy::input_focus::InputFocusPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::ui_widgets::Activate;
use bevy_persistent::StorageFormat;

use crate::audio::AudioCue;
use crate::render::LowRes;
use crate::saves::{SaveDir, SavedFlash};
use crate::settings::{Persistent, Settings};
use crate::ship::ShipTelemetry;

/// Returns an empty scratch directory of its own for one test.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("abstractions-menus-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Returns default settings stored in the scratch directory of one test.
fn stored(dir: &std::path::Path) -> Persistent<Settings> {
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(dir.join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap()
}

/// Inserts what the UI reads from the other modules: settings, the save directory, the render
/// target, the ship instruments, the physics clock and the sound cues,
/// which are kept for the tests to read.
fn stubs(app: &mut App, dir: &std::path::Path) {
    let frame = Duration::from_millis(16);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(frame))
        .insert_resource(stored(dir))
        .insert_resource(SaveDir(dir.join("worlds")))
        .insert_resource(LowRes(Handle::default()))
        .init_resource::<SavedFlash>()
        .init_resource::<ShipTelemetry>()
        .init_resource::<Time<Physics>>()
        .add_message::<AudioCue>()
        .init_resource::<heard::Heard>()
        .add_systems(Last, heard::hear);
}

/// Builds a headless app with the screens, the menus, the UI and the board computer, at sixty
/// frames per second. It starts on the main menu.
fn app(tag: &str) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        StatesPlugin,
        InputPlugin,
        InputFocusPlugin,
    ));
    stubs(&mut app, &scratch(tag));
    app.add_plugins((
        crate::screens::plugin,
        crate::menus::plugin,
        crate::ui::plugin,
        crate::computer::plugin,
        crate::net::plugin,
    ));
    for _ in 0..3 {
        app.update();
    }
    app
}

/// Presses and releases one key, with the text it types, over two frames.
fn press(app: &mut App, code: KeyCode, logical: Key, text: Option<&str>) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code: code,
            logical_key: logical.clone(),
            state,
            text: text
                .filter(|_| state == ButtonState::Pressed)
                .map(Into::into),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
}

/// Presses and releases a key that types nothing.
fn tap(app: &mut App, code: KeyCode) {
    let logical = match code {
        KeyCode::Enter => Key::Enter,
        KeyCode::Escape => Key::Escape,
        KeyCode::Backspace => Key::Backspace,
        _ => Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
    };
    press(app, code, logical, None);
}

/// Activates the only entity that carries exactly this component value, then runs two frames.
fn activate<C: Component + PartialEq>(app: &mut App, wanted: C) {
    let mut query = app.world_mut().query::<(Entity, &C)>();
    let found: Vec<Entity> = query
        .iter(app.world())
        .filter(|(_, value)| **value == wanted)
        .map(|(entity, _)| entity)
        .collect();
    assert_eq!(found.len(), 1, "one button carries the component");
    app.world_mut().trigger(Activate { entity: found[0] });
    app.update();
    app.update();
}
