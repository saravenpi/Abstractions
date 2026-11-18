//! Saving the active world: periodic, on pause, on leaving and on exit.
//!
//! A save is captured on the main thread, which is cheap, and handed to [`super::Writer`], which
//! writes the file on the I/O task pool. No frame waits for the disk.

use bevy::prelude::*;

use super::snapshot::Snapshot;
use crate::menus::Menu;
use crate::screens::{Screen, playing};

/// Seconds of play between two automatic saves.
const INTERVAL: f32 = 60.0;

/// Seconds the "Saved" notice stays visible.
pub(super) const FLASH_SECONDS: f32 = 2.0;

/// Seconds left to show the "Saved" notice after a successful save.
#[derive(Resource, Debug, Default)]
pub struct SavedFlash(pub f32);

/// Counts play time until the next automatic save.
#[derive(Resource)]
struct AutoTimer(Timer);

/// The systems that write the active world: on pause, on leaving, on exit and every
/// interval. Code that must change the world before it is saved runs before this set.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SaveSet;

/// Saves without waiting for the disk.
fn save_now(mut snapshot: Snapshot) {
    snapshot.save();
}

/// Saves when the application is about to exit, and waits for the file.
fn save_on_exit(mut exits: MessageReader<AppExit>, mut snapshot: Snapshot) {
    if exits.read().next().is_some() {
        snapshot.save_and_wait();
    }
}

/// Counts play time and saves every interval.
fn tick(time: Res<Time>, mut timer: ResMut<AutoTimer>, mut snapshot: Snapshot) {
    if let Some(active) = snapshot.active.as_deref_mut() {
        active.0.save.play_seconds += f64::from(time.delta_secs());
    }
    if timer.0.tick(time.delta()).just_finished() {
        snapshot.save();
    }
}

/// Counts the "Saved" notice down.
fn tick_flash(time: Res<Time>, mut flash: ResMut<SavedFlash>) {
    if flash.0 > 0.0 {
        flash.0 = (flash.0 - time.delta_secs()).max(0.0);
    }
}

/// Registers the autosave systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SavedFlash>()
        .insert_resource(AutoTimer(Timer::from_seconds(
            INTERVAL,
            TimerMode::Repeating,
        )))
        .add_systems(OnEnter(Menu::Pause), save_now.in_set(SaveSet))
        .add_systems(OnExit(Screen::Gameplay), save_now.in_set(SaveSet))
        .add_systems(Update, (tick.run_if(playing).in_set(SaveSet), tick_flash))
        .add_systems(Last, save_on_exit.in_set(SaveSet));
}
