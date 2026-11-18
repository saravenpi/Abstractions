//! Weather: clouds, rain, snow, dust and storms at the camera, from the climate there.
//!
//! The weather is a pure function of the climate at the camera, the seed of the planet and
//! universe time. The particles, the wind and the sound read it from [`Weather`].

mod feed;
mod noise;
mod rules;
mod watch;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_time;

pub use feed::word;
pub use rules::{THUNDER_SECONDS, Weather, thunder};

use bevy::prelude::*;

use crate::screens::Screen;

/// The system that reads the weather at the camera. What follows the weather runs after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Forecast;

/// Registers the weather resource, the system that keeps it at the camera, and the one that
/// hands it to the sky and the HUD.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Weather>()
        .init_resource::<crate::menus::HudWeather>()
        .init_resource::<crate::render::LocalWeather>()
        .add_systems(OnExit(Screen::Gameplay), watch::clear)
        .add_systems(
            Update,
            (watch::watch, feed::feed)
                .chain()
                .in_set(Forecast)
                .run_if(in_state(Screen::Gameplay)),
        );
}
