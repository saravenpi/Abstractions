//! Saving the active world: periodic, on pause, on leaving and on exit.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::store::{now, write};
use super::{ActiveWorld, SaveDir};
use crate::menus::Menu;
use crate::player::Player;
use crate::render::{DayClock, GameCamera};
use crate::screens::{Screen, playing};

/// Seconds of play between two automatic saves.
const INTERVAL: f32 = 60.0;

/// Seconds the "Saved" notice stays visible.
const FLASH_SECONDS: f32 = 2.0;

/// Seconds left to show the "Saved" notice after a successful save.
#[derive(Resource, Debug, Default)]
pub struct SavedFlash(pub f32);

/// Counts play time until the next automatic save.
#[derive(Resource)]
struct AutoTimer(Timer);

/// Everything needed to capture the world state and write it.
#[derive(SystemParam)]
struct Snapshot<'w, 's> {
    dir: Res<'w, SaveDir>,
    active: Option<ResMut<'w, ActiveWorld>>,
    clock: Res<'w, DayClock>,
    flash: ResMut<'w, SavedFlash>,
    player: Query<'w, 's, &'static Transform, (With<Player>, Without<GameCamera>)>,
    camera: Query<'w, 's, &'static Transform, (With<GameCamera>, Without<Player>)>,
}

impl Snapshot<'_, '_> {
    /// Captures the pose and clock into the active world and writes its file.
    fn save(&mut self) {
        let Some(active) = self.active.as_deref_mut() else {
            return;
        };
        let save = &mut active.0.save;
        if let (Ok(body), Ok(eyes)) = (self.player.single(), self.camera.single()) {
            save.position = Some(body.translation.to_array());
            save.yaw = body.rotation.to_euler(EulerRot::YXZ).0;
            save.pitch = eyes.rotation.to_euler(EulerRot::YXZ).1;
        }
        save.day_phase = self.clock.phase;
        save.last_played = now();
        match write(&self.dir.0, &active.0.id, save) {
            Ok(()) => self.flash.0 = FLASH_SECONDS,
            Err(error) => warn!("could not save world {}: {error}", active.0.id),
        }
    }
}

/// Saves immediately.
fn save_now(mut snapshot: Snapshot) {
    snapshot.save();
}

/// Saves when the application is about to exit.
fn save_on_exit(mut exits: MessageReader<AppExit>, mut snapshot: Snapshot) {
    if exits.read().next().is_some() {
        snapshot.save();
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
        .add_systems(OnEnter(Menu::Pause), save_now)
        .add_systems(OnExit(Screen::Gameplay), save_now)
        .add_systems(Update, (tick.run_if(playing), tick_flash))
        .add_systems(Last, save_on_exit);
}
