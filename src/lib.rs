//! Abstractions: a pixelated low-poly island exploration game.

pub mod audio;
pub mod companion;
pub mod computer;
pub mod effects;
pub mod menus;
pub mod player;
pub mod render;
pub mod saves;
pub mod screens;
pub mod settings;
pub mod ship;
pub mod space;
pub mod ui;
pub mod universe;
pub mod wind;
pub mod world;

use avian3d::prelude::*;
use bevy::prelude::*;

/// Root plugin wiring every game module together.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        let mut saved = settings::load();
        let start = render::ray_tracing::begin(&mut saved);
        let defaults = DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Abstractions".into(),
                ..default()
            }),
            ..default()
        });
        app.insert_resource(saved).add_plugins(defaults);
        render::ray_tracing::prepare(app, start);
        app.add_plugins(PhysicsPlugins::default()).add_plugins((
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
    }
}
