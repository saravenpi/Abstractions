//! The updater inside the game loop: one check at the title screen, and reading the threads.

use bevy::prelude::*;

use super::install::sweep;
use super::updates::Updates;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};

/// Once per launch, at the title screen: removes what the last update left behind, since the
/// game has started, and looks for a newer release when the setting asks for it.
fn launch(
    mut done: Local<bool>,
    settings: Res<Persistent<Settings>>,
    mut updates: ResMut<Updates>,
) {
    if std::mem::replace(&mut *done, true) {
        return;
    }
    sweep(&updates.place.exe);
    if settings.updates.check_at_launch {
        updates.check();
    }
}

/// Takes in what the background threads reported. The updater counts as changed only then.
fn poll(mut updates: ResMut<Updates>) {
    if updates.bypass_change_detection().poll() {
        updates.set_changed();
    }
}

/// Cancels a running download when the player leaves the title screen: nothing downloads
/// while a world is loaded.
fn leave(mut updates: ResMut<Updates>) {
    updates.cancel();
}

/// Registers the updater systems. The [`Updates`] resource comes from the caller.
pub fn systems(app: &mut App) {
    app.init_resource::<Updates>()
        .add_systems(
            Update,
            (launch.run_if(in_state(Screen::Title)), poll).chain(),
        )
        .add_systems(OnExit(Screen::Title), leave);
}
