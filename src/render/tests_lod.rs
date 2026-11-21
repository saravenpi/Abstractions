//! Tests that the shadows are cast with the level of detail the game camera sees: the game camera
//! is the shadow LOD origin, not the interface camera that draws the picture to the window.
//!
//! Without it Bevy measured the levels of the shadow pass from the interface camera, which stands
//! at the render origin: the robot, the companion and the ship, whose near mesh shows only close to
//! a camera, cast no shadow once they stood away from that origin.

use bevy::camera::{RenderTarget, ShadowLodOrigin};
use bevy::ecs::system::RunSystemOnce;
use bevy::light::get_shadow_lod_origin;
use bevy::prelude::*;
use bevy::window::WindowRef;
use bevy_persistent::StorageFormat;

use super::GameCamera;
use super::palette::Palette;
use crate::settings::{Persistent, Settings};

/// Returns default settings stored in a scratch file of their own.
fn settings() -> Persistent<Settings> {
    let dir = format!("abstractions-lod-test-{}", std::process::id());
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(std::env::temp_dir().join(dir).join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap()
}

/// The point and spot lights.
type Lamps = Or<(With<PointLight>, With<SpotLight>)>;

/// Returns the entity Bevy measures the levels of detail of the shadow passes from.
fn origin(
    mut cameras: Query<(Entity, &RenderTarget), With<Camera>>,
    mut marked: Query<Entity, With<ShadowLodOrigin>>,
    mut lights: Query<Entity, Lamps>,
) -> Option<Entity> {
    get_shadow_lod_origin(
        cameras.transmute_lens_filtered(),
        marked.transmute_lens_filtered(),
        lights.transmute_lens_filtered(),
    )
}

/// With the interface camera drawing to the window at the render origin, as the game has it, and
/// the game camera standing 40 m away by the robot, the shadows are measured from the game camera.
#[test]
fn the_game_camera_is_the_shadow_lod_origin() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(settings())
        .insert_resource(Palette::at(0.25));
    super::camera::plugin(&mut app);
    let window = RenderTarget::Window(WindowRef::Primary);
    app.world_mut()
        .spawn((Camera2d, Camera::default(), window, Transform::default()));
    let game = app
        .world_mut()
        .spawn((
            GameCamera,
            Camera3d::default(),
            Transform::from_xyz(40.0, 2.0, -12.0),
        ))
        .id();
    app.update();
    assert!(app.world().entity(game).contains::<ShadowLodOrigin>());
    let chosen = app.world_mut().run_system_once(origin).unwrap();
    assert_eq!(chosen, Some(game));
}
