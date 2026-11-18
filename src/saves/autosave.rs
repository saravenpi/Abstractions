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
use crate::player::{Player, local};
use crate::render::DayClock;
use crate::screens::{Screen, playing};
use crate::ship::Logbook;
use crate::sim::PlayerId;
use crate::view::Heading;

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

/// The body of the local player and the look angles of the camera rig.
#[derive(SystemParam)]
struct Poses<'w, 's> {
    /// The player bodies.
    bodies: Query<'w, 's, (&'static PlayerId, &'static Transform), With<Player>>,
    /// The look angles: on foot the heading and the pitch of the view.
    heading: Res<'w, Heading>,
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
    /// The heading and the pitch are those the camera rig holds, never read back from a
    /// transform, so a third person camera saves the same angles as the eyes. Seated in the
    /// ship the view angles on foot are kept.
    fn capture(&mut self) -> Option<&ActiveWorld> {
        let active = self.active.as_deref_mut()?;
        let save = &mut active.0.save;
        let bodies = self.poses.bodies.iter().map(|(id, body)| (*id, body));
        if let Some(body) = local(bodies) {
            self.site.stand(save, body.translation);
            if self.site.walking() {
                save.yaw = self.poses.heading.yaw;
                save.pitch = self.poses.heading.pitch;
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
