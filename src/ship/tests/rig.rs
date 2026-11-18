//! A whole game without a window: every gameplay plugin that runs without a GPU, on a manual
//! clock, driven through the keyboard the way a player drives it.

mod drive;
mod look;
mod sight;
mod worlds;

use std::path::PathBuf;
use std::time::Duration;

use avian3d::prelude::PhysicsPlugins;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::input::InputPlugin;
use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy_persistent::StorageFormat;

use crate::audio::AudioCue;
use crate::companion::CompanionChirp;
use crate::menus::Menu;
use crate::saves::SaveDir;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::ship::ShipEvent;
use crate::wind::Wind;
use crate::world::WorldReady;

/// Seconds of game time in one frame of the rig while the player is on foot.
pub const FRAME: f64 = 1.0 / 30.0;

/// Seconds of game time in one frame while the player flies: the longest frame the ship takes
/// whole. The flight itself always advances in its own fixed steps, so only the test is faster.
pub const FLIGHT: f64 = 0.1;

/// Everything that was announced while the game ran, in order.
#[derive(Resource, Debug, Default, Clone)]
pub struct Log {
    /// What happened to the ship.
    pub ship: Vec<ShipEvent>,
    /// The sounds that were asked for.
    pub cues: Vec<AudioCue>,
    /// What the companion said.
    pub chirps: Vec<CompanionChirp>,
}

/// A running game and the scratch directory it keeps its files in.
pub struct Game {
    /// The application.
    pub app: App,
    /// Directory of the settings file and of the world files.
    pub dir: PathBuf,
}

/// Returns default settings stored in `dir`.
pub fn settings(dir: &std::path::Path) -> Persistent<Settings> {
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(dir.join("settings.toml"))
        .default(Settings::default())
        .build()
        .expect("settings in a scratch directory")
}

/// Copies every announcement of the frame into the log.
fn record(
    mut log: ResMut<Log>,
    mut ship: MessageReader<ShipEvent>,
    mut cues: MessageReader<AudioCue>,
    mut chirps: MessageReader<CompanionChirp>,
) {
    log.ship.extend(ship.read().copied());
    log.cues.extend(cues.read().copied());
    log.chirps.extend(chirps.read().copied());
}

/// Moves from the loading screen to the game once the world is ready, as the loading screen does.
fn start_playing(ready: Option<Res<WorldReady>>, mut screen: ResMut<NextState<Screen>>) {
    if ready.is_some_and(|ready| ready.0) {
        screen.set(Screen::Gameplay);
    }
}

/// Adds the engine parts a game needs without a window: time, assets, states, transforms,
/// input, audio without a device and physics.
fn engine(app: &mut App, dir: &std::path::Path) {
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        StatesPlugin,
        TransformPlugin,
        InputPlugin,
        AudioPlugin::default(),
    ))
    .add_plugins(PhysicsPlugins::default())
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        FRAME,
    )))
    .insert_resource(settings(dir))
    .insert_resource(SaveDir(dir.join("worlds")))
    .init_asset::<Mesh>()
    .init_asset::<Image>()
    .init_asset::<StandardMaterial>()
    .init_resource::<GlobalAmbientLight>()
    .init_resource::<ClearColor>()
    .init_resource::<Wind>()
    .init_resource::<Log>()
    .init_state::<Screen>()
    .init_state::<Menu>();
}

impl Game {
    /// Builds the game on the title screen, with its files in a scratch directory named after
    /// `tag`. The body, the flight gear and the camera rig are in, the rig driving the game
    /// camera as it does in the game.
    pub fn full(tag: &str) -> Self {
        let name = format!("abstractions-voyage-{tag}-{}", std::process::id());
        let dir = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&dir);
        let mut app = App::new();
        engine(&mut app, &dir);
        app.add_plugins((
            crate::space::plugin,
            crate::world::plugin,
            crate::player::plugin,
            crate::render::sky,
            crate::saves::plugin,
            crate::ship::plugin,
            crate::computer::logic,
            crate::audio::plugin,
            crate::companion::plugin,
            crate::home::plugin,
        ))
        .add_systems(Update, start_playing.run_if(in_state(Screen::Loading)))
        .add_systems(Last, record);
        app.add_plugins((
            crate::avatar::plugin,
            crate::gear::plugin,
            crate::view::plugin,
        ))
        .insert_resource(crate::view::Enabled(true));
        app.finish();
        app.cleanup();
        Self { app, dir }
    }
}

impl Drop for Game {
    /// Removes the scratch directory.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
