//! Self-update: looking for a newer release on GitHub, and installing it when the player asks.
//!
//! The game checks at most once per launch, at the title screen, and when the player asks from
//! the main menu. Nothing is downloaded before the player chooses to. The check and the download
//! run on background threads; menus read [`Updates`].
//!
//! Pure rules are in plain files: [`version`], [`release`], [`platform`], [`allow`],
//! [`checksum`], [`install`], [`download`], [`job`]. The network sits behind the [`Host`] trait,
//! so tests run the whole path with a host of their own and never touch the network.
//!
//! Integrity: a download must have the size the release declares and the SHA-256 its checksum
//! list gives. Authenticity rests on HTTPS to GitHub alone: the builds are not signed.

pub mod allow;
pub mod check;
pub mod checksum;
pub mod download;
mod failure;
#[cfg(test)]
pub mod fake;
#[cfg(test)]
pub mod fixture;
mod host;
pub mod install;
pub mod job;
mod net;
mod open;
pub mod place;
pub mod platform;
mod prefs;
pub mod release;
mod run;
mod signals;
mod state;
mod systems;
#[cfg(test)]
mod tests;
mod updates;
pub mod version;

pub use failure::Failure;
pub use host::{Host, Offline, Sink};
pub use open::PAGE;
pub use place::Place;
pub use platform::Platform;
pub use prefs::UpdatePrefs;
pub use release::{Asset, Release};
pub use signals::Signals;
pub use state::{Outcome, Plan, Stage, Status};
pub use systems::systems;
pub use updates::Updates;

use std::sync::Arc;

use bevy::prelude::*;

/// Gives the game an updater that reaches GitHub, and registers its systems.
pub(super) fn plugin(app: &mut App) {
    let place = Place::current();
    let updates = Updates::new(Arc::new(net::Net), place, env!("CARGO_PKG_VERSION"));
    app.insert_resource(updates);
    systems(app);
}
