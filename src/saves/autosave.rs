//! Saving the active world: periodic, on pause, on leaving and on exit.
//!
//! A save is captured on the main thread, which is cheap, and handed to [`Writer`], which
//! writes the file on the I/O task pool. No frame waits for the disk.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::capture::Site;
use super::store::now;
use super::writer::Writer;
use super::{ActiveWorld, SaveDir};
use crate::menus::Menu;
use crate::player::{Motion, Player};
use crate::render::{DayClock, GameCamera};
use crate::screens::{Screen, playing};
use crate::ship::Logbook;

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

/// The body and the eyes of the player.
#[derive(SystemParam)]
struct Poses<'w, 's> {
    /// The body, with the heading it stores.
    body: Query<'w, 's, (&'static Transform, &'static Motion), With<Player>>,
    /// The camera, a child of the body that only looks up and down.
    eyes: Query<'w, 's, &'static Transform, (With<GameCamera>, Without<Player>)>,
}

/// Everything needed to capture the world state and hand it to the writer.
#[derive(SystemParam)]
struct Snapshot<'w, 's> {
    /// Where the world files are.
    dir: Res<'w, SaveDir>,
    /// The world being played.
    active: Option<ResMut<'w, ActiveWorld>>,
    /// The time of day shown.
    clock: Res<'w, DayClock>,
    /// Writes the files off the main thread.
    writer: ResMut<'w, Writer>,
    /// Where and when the player is.
    site: Site<'w>,
    /// The body and the eyes of the player.
    poses: Poses<'w, 's>,
    /// The record of the ship.
    ship: Logbook<'w, 's>,
}

impl Snapshot<'_, '_> {
    /// Captures the pose, the ship, the clocks and the atlas into the active world.
    ///
    /// The heading is the stored one of the body, never read back from its orientation. Seated
    /// in the ship the view angles on foot are kept.
    fn capture(&mut self) -> Option<&ActiveWorld> {
        let active = self.active.as_deref_mut()?;
        let save = &mut active.0.save;
        if let (Ok((body, motion)), Ok(eyes)) = (self.poses.body.single(), self.poses.eyes.single())
        {
            self.site.stand(save, body.translation);
            if self.site.walking() {
                save.yaw = motion.yaw;
                save.pitch = eyes.rotation.to_euler(EulerRot::YXZ).1;
            }
        }
        if let Some(ship) = self.ship.record() {
            save.ship = Some(ship);
        }
        self.site.record(save);
        save.day_phase = self.clock.phase;
        save.last_played = now();
        Some(active)
    }

    /// Captures the world and queues its file to be written off the main thread.
    fn save(&mut self) {
        if let Some(world) = self.capture().cloned() {
            self.writer.queue(world);
        }
    }

    /// Captures the world and writes its file before returning. Only for the last frame of the
    /// application, when no later frame could finish a queued write.
    fn save_and_wait(&mut self) {
        self.save();
        let dir = self.dir.0.clone();
        self.writer.finish(&dir);
    }
}

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
        .add_systems(OnEnter(Menu::Pause), save_now)
        .add_systems(OnExit(Screen::Gameplay), save_now)
        .add_systems(Update, (tick.run_if(playing), tick_flash))
        .add_systems(Last, save_on_exit);
}
