//! Stopping a world open to LAN: on Close LAN, on quitting to the title and on quitting the
//! game. The room's log is merged back and the world is saved as in single player.

use abstractions_net::store::World;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Hosted;
use crate::computer::Atlas;
use crate::net::join::{merge, shut};
use crate::net::link::Link;
use crate::net::{CloseLan, Players, Session};
use crate::saves::Snapshot;
use crate::sim::Changes;

/// What stopping to host changes.
#[derive(SystemParam)]
pub struct Closing<'w, 's> {
    /// Removes the server and the link.
    commands: Commands<'w, 's>,
    /// The server and its beacon.
    hosted: ResMut<'w, Hosted>,
    /// This game's own link to its server.
    link: Option<ResMut<'w, Link>>,
    /// The log of the world.
    changes: Option<ResMut<'w, Changes>>,
    /// What the player discovered.
    atlas: Option<ResMut<'w, Atlas>>,
    /// The players of the session, forgotten.
    players: ResMut<'w, Players>,
}

impl Closing<'_, '_> {
    /// Stops the beacon, leaves the own server, stops it, merges what its room logged into
    /// the world and goes back to single player.
    fn stop(&mut self) {
        if let Some(announcer) = self.hosted.announcer.take() {
            announcer.stop();
        }
        if let Some(link) = self.link.as_deref_mut() {
            link.close(false);
        }
        let now = self.hosted.started.elapsed().as_secs_f64();
        if let Some(server) = self.hosted.server.take() {
            self.adopt(&server.stop().snapshot(now));
        }
        shut(&mut self.commands);
        self.commands.remove_resource::<Hosted>();
        self.commands.insert_resource(Session::Offline);
        self.players.0.clear();
    }

    /// Appends each change of `world` the log does not hold yet, and adds every visited body
    /// to the atlas.
    fn adopt(&mut self, world: &World) {
        let Some(changes) = self.changes.as_deref_mut() else {
            return;
        };
        for entry in &world.changes.entries {
            if changes.entries.iter().all(|old| old.change != entry.change) {
                changes.entries.push(entry.clone());
            }
        }
        if let Some(atlas) = self.atlas.as_deref_mut() {
            merge(atlas, changes);
        }
    }
}

/// Marks a world whose hosting stopped during play: it is saved on the next step.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Resave;

/// Stops hosting on Close LAN; the world is saved right after.
pub fn close_lan(mut closes: MessageReader<CloseLan>, mut closing: Closing) {
    if closes.read().last().is_some() {
        closing.stop();
        closing.commands.insert_resource(Resave);
    }
}

/// Saves the world whose hosting just stopped, with the log of the room merged in.
pub fn resave(mut snapshot: Snapshot, mut commands: Commands) {
    snapshot.save();
    commands.remove_resource::<Resave>();
}

/// Stops hosting as the player leaves the world, before it is saved.
pub fn close_on_leave(mut closing: Closing) {
    closing.stop();
}

/// Stops hosting when the game quits, before the world is saved.
pub fn close_on_exit(mut exits: MessageReader<AppExit>, mut closing: Closing) {
    if exits.read().next().is_some() {
        closing.stop();
    }
}
