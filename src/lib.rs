//! Abstractions: a pixelated low-poly island exploration game.

pub mod menus;
pub mod player;
pub mod render;
pub mod saves;
pub mod screens;
pub mod settings;
pub mod ui;
pub mod world;

use avian3d::prelude::*;
use bevy::prelude::*;

/// Root plugin wiring every game module together.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Abstractions".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins((
            settings::plugin,
            screens::plugin,
            menus::plugin,
            ui::plugin,
            world::plugin,
            player::plugin,
            render::plugin,
            saves::plugin,
        ));
    }
}
