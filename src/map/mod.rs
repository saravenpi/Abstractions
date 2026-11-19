//! The map: one screen that zooms from the ground under the player to the whole universe.
//!
//! - [`explored`] keeps the ground each body has had uncovered and shares it on a server.
//! - [`raster`] paints the flat coloured picture of a body, black where nobody has been.

pub mod explored;
pub mod raster;

use bevy::prelude::*;

/// Registers the map.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((explored::plugin, raster::plugin));
}
