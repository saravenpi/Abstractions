//! The minimap: a small square in the top right corner that shows the ground around the
//! player on a planet, heading up, and the chart of the system in space.
//!
//! On a planet the ground is painted into a small image fixed to the planet and wider than the
//! view, off the main thread, so that walking and turning only move and turn that image. The
//! home base, the ship, friends, waypoints and favourites are marks that wait on the rim while
//! they are out of range. In space
//! the board computer lends its chart and its marks. The minimap is part of the HUD, so it
//! hides behind every menu and behind the computer, which has the large map.

mod dots;
mod follow;
mod marks;
mod orbit;
mod paint;
mod patch;
mod pose;
mod range;
mod rings;
mod scope;
mod sight;
mod space;
#[cfg(test)]
mod tests;
mod view;
pub(in crate::menus::hud) mod widget;

pub(super) use pose::Reader;
pub(super) use widget::frame;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use super::MarkerSet;
use crate::screens::{Screen, in_control};
use crate::settings::{Persistent, Settings};

/// Seconds between two looks at the system chart.
const CHART_EVERY: f32 = 0.2;

/// Moves the minimap to its next mode when its key is pressed: the range follows the speed,
/// then each fixed range from the nearest, then off.
fn cycle(
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<Persistent<Settings>>,
    mut scope: ResMut<scope::Scope>,
) {
    if keys.just_pressed(settings.controls.keys.minimap) {
        scope.mode = scope.mode.next();
    }
}

/// Registers the minimap: the ground image is kept up to date while a world is on screen, and
/// the view is laid out while the player controls the game.
pub(super) fn plugin(app: &mut App) {
    let chart = space::chart.run_if(on_timer(Duration::from_secs_f32(CHART_EVERY)));
    let ground = (scope::fresh, scope::track, scope::refill, scope::collect);
    let shown = (
        cycle,
        marks::place,
        dots::place,
        marks::caption,
        chart,
        follow::follow,
        space::steer,
    );
    app.init_resource::<scope::Scope>()
        .init_resource::<follow::Charted>()
        .add_systems(
            Update,
            (ground.chain(), shown.chain().run_if(in_control))
                .chain()
                .in_set(MarkerSet::Show)
                .run_if(in_state(Screen::Gameplay)),
        );
}
