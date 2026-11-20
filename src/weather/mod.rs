//! Weather: clouds, rain, snow, dust and storms, from the cloud field of the planet.
//!
//! The cloud field is a pure function of the seed of the planet, a place on it and universe
//! time. The sky draws it, and the weather at the camera is read from it and from the climate
//! there. The particles, the wind and the sound read that weather from [`Weather`].

mod feed;
mod field;
mod grid;
mod noise;
mod puff;
mod rules;
mod watch;
mod wet;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_field;
#[cfg(test)]
mod tests_grid;
#[cfg(test)]
mod tests_ground;
#[cfg(test)]
mod tests_puff;
#[cfg(test)]
mod tests_time;

pub use feed::{seen, word};
pub use field::Field;
pub use grid::{FACE, Patch, SNAPSHOT, TEXELS, face_dir, moment, texel};
pub use puff::{CELL, density_at, drift};
pub use rules::{THUNDER_SECONDS, Weather, thunder};
pub use wet::wet;

use bevy::prelude::*;

use crate::screens::Screen;

/// The system that reads the weather at the camera. What follows the weather runs after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Forecast;

/// Registers the weather resource, the system that keeps it at the camera, and the one that
/// hands it to the sky and the HUD. The weather owns [`Weather`] and the word of the HUD; the
/// sky owns `render::LocalWeather`, which the weather only writes.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Weather>()
        .init_resource::<crate::menus::HudWeather>()
        .add_systems(OnExit(Screen::Gameplay), watch::clear)
        .add_systems(
            Update,
            (watch::watch, feed::feed)
                .chain()
                .in_set(Forecast)
                .run_if(in_state(Screen::Gameplay)),
        );
}
