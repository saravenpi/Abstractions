//! The in-game HUD: flight instruments in the ship, the interaction prompt, the location
//! banner, the compass that points home and to the ship, and the minimap. Everything hides
//! behind menus and the computer.

mod banner;
mod climate;
mod compass;
mod flight;
mod minimap;
mod needle;
mod panel;
mod prompt;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_climate;
#[cfg(test)]
mod tests_deck;
mod units;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::screens::{Screen, in_control};
use crate::ship::seated;
use crate::space::{Location, Travel};

/// Seconds between two refreshes of the flight instruments.
const REFRESH: f32 = 0.1;

/// Seconds between two refreshes of the climate line.
const CLIMATE_EVERY: f32 = 0.5;

/// Marks the root of the HUD.
#[derive(Component)]
struct HudRoot;

/// Spawns the hidden HUD for the world being entered.
fn spawn(mut commands: Commands) {
    commands.insert_resource(banner::Banner::default());
    commands.spawn((
        HudRoot,
        DespawnOnExit(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        Pickable::IGNORE,
        Visibility::Hidden,
        children![
            panel::block(),
            prompt::line(),
            banner::block(),
            compass::bar(),
            climate::line(),
            minimap::frame()
        ],
    ));
}

/// Shows the HUD while the player controls the game, and the instruments while seated.
fn show(
    In(control): In<bool>,
    location: Option<Res<Location>>,
    mut roots: Query<&mut Visibility, With<HudRoot>>,
    mut blocks: Query<&mut Node, With<panel::Instruments>>,
) {
    let wanted = match control {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
    };
    for mut visibility in &mut roots {
        visibility.set_if_neq(wanted);
    }
    let seated = location.is_some_and(|location| location.travel == Travel::InShip);
    for mut node in &mut blocks {
        let display = if seated { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
}

/// Registers the HUD systems.
pub(super) fn plugin(app: &mut App) {
    let every = on_timer(Duration::from_secs_f32(REFRESH));
    app.init_resource::<banner::Banner>()
        .add_plugins(minimap::plugin)
        .add_systems(OnEnter(Screen::Gameplay), spawn)
        .add_systems(
            Update,
            (
                in_control.pipe(show),
                (panel::refresh, panel::spool)
                    .run_if(in_control)
                    .run_if(seated)
                    .run_if(every),
                prompt::update.run_if(in_control),
                compass::update.run_if(in_control),
                climate::update
                    .run_if(in_control)
                    .run_if(on_timer(Duration::from_secs_f32(CLIMATE_EVERY))),
                banner::watch.run_if(in_control),
                banner::fade.run_if(banner::showing),
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
}
