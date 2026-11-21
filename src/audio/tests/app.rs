//! The plugin inside a headless app: it starts with or without an audio device and senses a world.

use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy_persistent::StorageFormat;

use crate::audio::feed::Bridge;
use crate::audio::shared::Param;
use crate::audio::{AudioCue, AudioScene};
use crate::menus::Menu;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::render::{DayClock, GameCamera};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Frame, Location, Travel};
use crate::universe::Universe;
use crate::wind::Wind;
use crate::world::Terrain;

/// Returns default settings stored in a scratch file of their own.
fn settings(name: &str) -> Persistent<Settings> {
    let dir = format!("abstractions-audio-test-{}-{name}", std::process::id());
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(std::env::temp_dir().join(dir).join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap()
}

/// Builds a headless app with the audio plugin and what it reads, at sixty frames per second.
pub(super) fn app(name: &str) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )))
    .insert_resource(settings(name))
    .init_resource::<DayClock>()
    .init_resource::<Wind>()
    .init_state::<Screen>()
    .init_state::<Menu>()
    .add_plugins(crate::audio::plugin);
    app
}

/// Puts a flat world, a player standing on its spawn and a camera in the app, and starts playing.
pub(super) fn enter_world(app: &mut App) -> Entity {
    let (universe, terrain) = (Universe::new(7), Terrain::new(7));
    let spawn = terrain.find_spawn();
    let feet = terrain.height(spawn.x, spawn.z) + 0.5 * BODY_LENGTH + BODY_RADIUS;
    let body = Vec3::new(spawn.x, feet, spawn.z);
    app.insert_resource(Anchor::surface(universe.home(), &terrain, body))
        .insert_resource(Location {
            body: universe.home(),
            travel: Travel::OnFoot,
        })
        .insert_resource(universe)
        .insert_resource(terrain)
        .insert_state(Screen::Gameplay);
    let eye = GlobalTransform::from_translation(body + Vec3::Y * 0.7);
    app.world_mut().spawn((GameCamera, eye));
    app.world_mut()
        .spawn((Player, Transform::from_translation(body)))
        .id()
}

/// Walks the player `frames` frames along the ground at walking pace and returns the cues written.
fn walk(app: &mut App, player: Entity, frames: usize) -> Vec<AudioCue> {
    let mut cues = Vec::new();
    for _ in 0..frames {
        let terrain = app.world().resource::<Terrain>().clone();
        let mut body = app.world_mut().get_mut::<Transform>(player).unwrap();
        body.translation.x += 0.08;
        let ground = terrain.height(body.translation.x, body.translation.z);
        body.translation.y = ground + 0.5 * BODY_LENGTH + BODY_RADIUS;
        app.update();
        cues.extend(app.world_mut().resource_mut::<Messages<AudioCue>>().drain());
    }
    cues
}

/// On the title screen the plugin runs, keeps the title scene and publishes the volume settings.
#[test]
fn plugin_runs_without_a_world() {
    let mut app = app("title");
    for _ in 0..4 {
        app.world_mut().write_message(AudioCue::UiConfirm);
        app.update();
    }
    assert_eq!(*app.world().resource::<AudioScene>(), AudioScene::default());
    let shared = app.world().resource::<Bridge>().0.clone();
    assert_eq!(shared.get(Param::Music), Settings::default().audio.music);
    assert_eq!(shared.get(Param::MusicLevel), 1.0);
    let mut speakers = app
        .world_mut()
        .query::<&AudioPlayer<crate::audio::source::Synth>>();
    assert_eq!(speakers.iter(app.world()).count(), 1);
}

/// Walking on the ground keeps the scene on the home planet at ground level. The footsteps come
/// from the feet of the body, which this app has none of: `avatar::tread::tests` hears them.
#[test]
fn walking_fills_the_scene() {
    let mut app = app("walk");
    let player = enter_world(&mut app);
    let cues = walk(&mut app, player, 150);
    let steps = cues
        .iter()
        .filter(|cue| matches!(cue, AudioCue::Footstep(_)));
    assert_eq!(steps.count(), 0, "cues {cues:?}");
    let (scene, universe) = (*app.world().resource::<AudioScene>(), Universe::new(7));
    assert_eq!(scene.planet_seed, universe.planet(universe.home()).seed);
    assert!(scene.home && !scene.in_space && scene.cabin == 0.0);
    assert!(scene.altitude_fade > 0.95 && scene.submerged == 0.0 && scene.cloud_depth == 0.0);
    assert_eq!(scene.music_level(), scene.altitude_fade);
}

/// In the ship there are no footsteps, in space the music is off, and the title resets the scene.
#[test]
fn ship_space_and_title_change_the_scene() {
    let mut app = app("ship");
    let player = enter_world(&mut app);
    app.world_mut().resource_mut::<Location>().travel = Travel::InShip;
    assert!(walk(&mut app, player, 60).is_empty());
    assert_eq!(app.world().resource::<AudioScene>().cabin, 1.0);
    let star = app.world().resource::<Location>().body.star;
    app.world_mut().resource_mut::<Anchor>().frame = Frame::System(star);
    app.update();
    let scene = *app.world().resource::<AudioScene>();
    assert!(scene.in_space && scene.altitude_fade == 0.0 && scene.music_level() == 0.0);
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Title);
    app.update();
    assert_eq!(*app.world().resource::<AudioScene>(), AudioScene::default());
}
