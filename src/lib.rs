//! Abstractions: a pixelated low-poly island exploration game.

pub mod audio;
pub mod avatar;
pub mod climate;
pub mod companion;
pub mod computer;
pub mod effects;
pub mod gear;
pub mod home;
pub mod icon;
pub mod menus;
pub mod player;
pub mod render;
pub mod saves;
pub mod screens;
pub mod settings;
pub mod ship;
pub mod sim;
pub mod space;
pub mod ui;
pub mod universe;
pub mod view;
pub mod wind;
pub mod world;

use avian3d::prelude::*;
use bevy::prelude::*;

/// Returns the main window: its title, its application name for Wayland, and its smallest size.
fn window() -> Window {
    Window {
        title: "Abstractions".into(),
        name: Some("abstractions".into()),
        resize_constraints: bevy::window::WindowResizeConstraints {
            min_width: 960.0,
            min_height: 540.0,
            ..default()
        },
        ..default()
    }
}

/// Adds every game module, with the camera rig driving the game camera.
fn modules(app: &mut App) {
    app.add_plugins((
        settings::plugin,
        screens::plugin,
        menus::plugin,
        ui::plugin,
        world::plugin,
        player::plugin,
        render::plugin,
        saves::plugin,
        wind::plugin,
        effects::plugin,
        space::plugin,
        ship::plugin,
        computer::plugin,
        audio::plugin,
        companion::plugin,
    ));
    app.add_plugins((
        home::plugin,
        avatar::plugin,
        gear::plugin,
        view::plugin,
        icon::plugin,
    ))
    .insert_resource(view::Enabled(true));
}

/// Root plugin wiring every game module together.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        let mut saved = settings::load();
        let start = render::ray_tracing::begin(&mut saved);
        let defaults = DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window()),
                ..default()
            })
            .set(render::ray_tracing::render_plugin());
        app.insert_resource(saved).add_plugins(defaults);
        render::ray_tracing::prepare(app, start);
        app.add_plugins(PhysicsPlugins::default());
        modules(app);
    }
}
