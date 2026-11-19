//! The harness of the headless minimap tests: an app with the screens, the menus, the HUD and
//! the board computer, a world on the home planet, and a camera to stand somewhere on it.

use std::sync::Arc;
use std::time::Duration;

use avian3d::prelude::Physics;
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::{ButtonState, InputPlugin};
use bevy::input_focus::InputFocusPlugin;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy_persistent::StorageFormat;

use crate::audio::AudioCue;
use crate::computer::Atlas;
use crate::menus::Menu;
use crate::menus::hud::minimap::scope::Scope;
use crate::render::{GameCamera, LowRes};
use crate::saves::{SaveDir, SavedFlash};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::ship::ShipTelemetry;
use crate::space::{Anchor, Clock, Location, Travel};
use crate::universe::Universe;
use crate::world::globe::Face;
use crate::world::{Globe, Planet, Terrain};

/// Radius in metres of the home planet, where the tests stand.
pub const RADIUS: f64 = 6000.0;

/// Returns the point of the ground `north` and `east` metres from the place the tests start
/// at, in the frame of the planet. The start is on the equator, where +Y is north.
pub fn ground(north: f64, east: f64) -> DVec3 {
    let (up, over) = ((north / RADIUS).tan(), (east / RADIUS).tan());
    (DVec3::X + DVec3::Y * up - DVec3::Z * over).normalize() * RADIUS
}

/// Returns default settings stored in a scratch directory of its own for the test `tag`.
fn settings(tag: &str) -> (std::path::PathBuf, Persistent<Settings>) {
    let name = format!("abstractions-minimap-{tag}-{}", std::process::id());
    let dir = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let stored = Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(dir.join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap();
    (dir, stored)
}

/// Inserts what the interface reads from the other modules, at sixty frames per second.
fn stubs(app: &mut App, tag: &str) {
    let (dir, stored) = settings(tag);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )))
    .insert_resource(stored)
    .insert_resource(SaveDir(dir.join("worlds")))
    .insert_resource(LowRes(Handle::default()))
    .init_resource::<SavedFlash>()
    .init_resource::<ShipTelemetry>()
    .init_resource::<Time<Physics>>()
    .add_message::<AudioCue>()
    .init_asset::<Image>();
}

/// Puts the home planet in `app` with the player on foot at the start, and starts playing.
fn world(app: &mut App) {
    let universe = Universe::new(7);
    let body = universe.home();
    let spec = universe.planet(body);
    let globe = Arc::new(Globe::of(&spec));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosX));
    let (travel, day_seconds) = (Travel::OnFoot, spec.day_seconds);
    app.insert_resource(Anchor::surface(body, &terrain, Vec3::ZERO))
        .insert_resource(Location { body, travel })
        .insert_resource(Planet {
            body,
            globe,
            day_seconds,
        })
        .insert_resource(Atlas::new(&universe))
        .insert_resource(Clock { seconds: 0.0 })
        .insert_resource(terrain)
        .insert_resource(universe);
    app.world_mut().spawn((GameCamera, Transform::default()));
    let world = app.world_mut();
    world
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Gameplay);
    world.resource_mut::<NextState<Menu>>().set(Menu::None);
}

/// Builds a headless app with the screens, the menus, the interface and the board computer,
/// playing on the home planet with the camera at the start, looking north.
pub fn app(tag: &str) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        StatesPlugin,
        InputPlugin,
        InputFocusPlugin,
    ));
    stubs(&mut app, tag);
    app.add_plugins((
        crate::screens::plugin,
        crate::menus::plugin,
        crate::ui::plugin,
        crate::computer::plugin,
        crate::net::plugin,
    ));
    app.update();
    world(&mut app);
    look(&mut app, ground(0.0, 0.0), DVec3::Y);
    app
}

/// Stands the camera at the point `at` of the planet, looking along `toward`, both in the
/// frame of the planet, and runs three frames.
pub fn look(app: &mut App, at: DVec3, toward: DVec3) {
    let anchor = *app.world().resource::<Anchor>();
    let turned = |along: DVec3| (anchor.rotation * along).as_vec3();
    let pose = Transform::from_translation(anchor.to_render(at));
    let pose = pose.looking_to(turned(toward), turned(at.normalize()));
    let mut cameras = app
        .world_mut()
        .query_filtered::<&mut Transform, With<GameCamera>>();
    for mut camera in cameras.iter_mut(app.world_mut()) {
        *camera = pose;
    }
    for _ in 0..3 {
        app.update();
    }
}

/// Presses and releases a key that types nothing, over two frames.
pub fn tap(app: &mut App, code: KeyCode) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code: code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
}

/// Runs frames until the worker has painted a patch for the range the map is on, waiting for
/// it as the game does, then until the shown range has come to rest. Returns that range.
pub fn settle(app: &mut App) -> f32 {
    for _ in 0..2000 {
        app.update();
        let scope = app.world().resource::<Scope>();
        let painted = scope.patch.is_some_and(|patch| patch.range == scope.radius);
        if painted && scope.job.is_none() {
            return scope.radius;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("no patch was painted");
}
