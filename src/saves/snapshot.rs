//! Capturing the active world for a save: the pose, the ship, the clocks and the atlas, taken on
//! the main thread and handed to [`Writer`].
//!
//! A world that lives on a server is captured too, so the network can send the player's place
//! and ship, but it is never handed to the writer: nothing of it reaches the local saves folder.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::capture::Site;
use super::store::now;
use super::writer::Writer;
use super::{ActiveWorld, SaveDir, Unsaved};
use crate::net::Session;
use crate::player::{Player, local};
use crate::render::DayClock;
use crate::ship::Logbook;
use crate::sim::PlayerId;
use crate::view::Heading;

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
pub struct Snapshot<'w, 's> {
    /// Where the world files are.
    dir: Res<'w, SaveDir>,
    /// The world being played.
    pub(super) active: Option<ResMut<'w, ActiveWorld>>,
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
    /// Present while the active world lives on a server.
    unsaved: Option<Res<'w, Unsaved>>,
    /// Where the game stands with other players.
    session: Option<Res<'w, Session>>,
}

impl Snapshot<'_, '_> {
    /// Captures the pose, the ship, the clocks and the atlas into the active world, and returns
    /// it. Nothing is written.
    ///
    /// The heading and the pitch are those the camera rig holds, never read back from a
    /// transform, so a third person camera saves the same angles as the eyes. Seated in the
    /// ship the view angles on foot are kept.
    pub fn capture(&mut self) -> Option<&ActiveWorld> {
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

    /// Returns true when the active world may be written to the local saves folder: it does
    /// not live on a server and the game is not joined to one.
    fn local(&self) -> bool {
        let client = self.session.as_deref().is_some_and(Session::client);
        self.unsaved.is_none() && !client
    }

    /// Captures the world and queues its file to be written off the main thread. A world that
    /// lives on a server is captured and not written.
    pub fn save(&mut self) {
        let local = self.local();
        if let Some(world) = self.capture().cloned().filter(|_| local) {
            self.writer.queue(world);
        }
    }

    /// Captures the world and writes its file before returning. Only for the last frame of the
    /// application, when no later frame could finish a queued write.
    pub(super) fn save_and_wait(&mut self) {
        self.save();
        let dir = self.dir.0.clone();
        self.writer.finish(&dir);
    }
}
