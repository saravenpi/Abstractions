//! Abstractions: a pixelated low-poly island exploration game.

pub mod audio;
pub mod avatar;
pub mod climate;
pub mod companion;
pub mod computer;
pub mod condition;
pub mod craft;
pub mod effects;
pub mod fauna;
pub mod gear;
pub mod home;
pub mod icon;
pub mod life;
pub mod map;
pub mod menus;
pub mod net;
pub mod player;
pub mod render;
pub mod saves;
pub mod screens;
pub mod settings;
pub mod ship;
pub mod sim;
pub mod space;
pub mod speed;
pub mod ui;
pub mod universe;
pub mod update;
pub mod view;
pub mod weather;
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
    later_modules(app);
}

/// Adds the modules that build on the ones above, with the camera rig driving the game camera.
fn later_modules(app: &mut App) {
    app.add_plugins((
        condition::plugin,
        home::plugin,
        avatar::plugin,
        gear::plugin,
        view::plugin,
        icon::plugin,
        sim::plugin,
        weather::plugin,
        update::plugin,
        net::plugin,
        net::driver::plugin,
        craft::plugin,
    ))
    .add_plugins(map::plugin)
    .insert_resource(view::Enabled(true));
}

/// Root plugin wiring every game module together.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        let saved = settings::load();
        let defaults = DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window()),
            ..default()
        });
        app.insert_resource(saved).add_plugins(defaults);
        app.add_plugins(PhysicsPlugins::default());
        modules(app);
        app.add_plugins(fauna::plugin);
    }
}
