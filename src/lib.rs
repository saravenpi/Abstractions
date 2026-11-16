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
        let mut saved = settings::load();
        let ray_tracing = render::ray_tracing::begin(&mut saved);
        let mut defaults = DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Abstractions".into(),
                ..default()
            }),
            ..default()
        });
        if ray_tracing {
            defaults = defaults.set(render::ray_tracing::render_plugin());
        }
        app.insert_resource(saved).add_plugins(defaults);
        if ray_tracing {
            render::ray_tracing::enable(app);
        }
        app.add_plugins(PhysicsPlugins::default()).add_plugins((
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
