//! Harness of the panel tests: the screens, the menus, the interface and the computer in an
//! app without a window, with the player seated in the ship of a loaded world.

use std::time::Duration;

use avian3d::prelude::Physics;
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::{ButtonState, InputPlugin};
use bevy::input_focus::InputFocusPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy_persistent::StorageFormat;

use super::Atlas;
use crate::audio::AudioCue;
use crate::menus::Menu;
use crate::render::LowRes;
use crate::saves::{SaveDir, SavedFlash};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::ship::ShipTelemetry;
use crate::space::{Anchor, Clock, Location, Travel};
use crate::universe::Universe;
use crate::world::Terrain;

/// Inserts what the screens and the interface read from the other modules, with the settings
/// stored in a scratch directory of this test.
fn stubs(app: &mut App, tag: &str) {
    let name = format!("abstractions-computer-{tag}-{}", std::process::id());
    let dir = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let settings = Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(dir.join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )))
    .insert_resource(settings)
    .insert_resource(SaveDir(dir.join("worlds")))
    .insert_resource(LowRes(Handle::default()))
    .init_resource::<SavedFlash>()
    .init_resource::<ShipTelemetry>()
    .init_resource::<Time<Physics>>()
    .add_message::<AudioCue>();
}

/// Puts a world in the app with the player seated in the ship, and starts playing.
fn seat(app: &mut App) {
    let (universe, terrain) = (Universe::new(7), Terrain::new(7));
    let body = universe.home();
    let travel = Travel::InShip;
    app.insert_resource(Anchor::surface(body, &terrain, Vec3::ZERO))
        .insert_resource(Location { body, travel })
        .insert_resource(Atlas::new(&universe))
        .insert_resource(Clock { seconds: 0.0 })
        .insert_resource(universe);
    let mut screen = app.world_mut().resource_mut::<NextState<Screen>>();
    screen.set(Screen::Gameplay);
    app.world_mut()
        .resource_mut::<NextState<Menu>>()
        .set(Menu::None);
}

/// Builds the app, seated in the ship of the home planet, a few frames into the game.
pub(super) fn game(tag: &str) -> App {
    let mut app = App::new();
    let core = (MinimalPlugins, AssetPlugin::default(), StatesPlugin);
    app.add_plugins((core, InputPlugin, InputFocusPlugin));
    stubs(&mut app, tag);
    let game = (
        crate::screens::plugin,
        crate::menus::plugin,
        crate::net::plugin,
    );
    app.add_plugins((game, crate::ui::plugin, super::plugin));
    run(&mut app, 3);
    seat(&mut app);
    run(&mut app, 3);
    app
}

/// Runs `frames` frames.
pub(super) fn run(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

/// Presses and releases one key over two frames. A key that types sends `text` with it.
pub(super) fn tap(app: &mut App, code: KeyCode, text: Option<&str>) {
    let logical = match (code, text) {
        (KeyCode::Escape, _) => Key::Escape,
        (KeyCode::Enter, _) => Key::Enter,
        (KeyCode::Backspace, _) => Key::Backspace,
        (_, Some(text)) => Key::Character(text.into()),
        _ => Key::Unidentified(NativeKey::Unidentified),
    };
    for state in [ButtonState::Pressed, ButtonState::Released] {
        let typed = text.filter(|_| state == ButtonState::Pressed);
        app.world_mut().write_message(KeyboardInput {
            key_code: code,
            logical_key: logical.clone(),
            state,
            text: typed.map(Into::into),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
}
