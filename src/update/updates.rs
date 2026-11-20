//! The updater as the game holds it: what is known, and the handles of the background work.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use super::failure::Failure;
use super::host::{Host, Offline};
use super::older::Older;
use super::place::Place;
use super::platform::pick;
use super::release::Release;
use super::signals::Signals;
use super::state::{Outcome, Plan, Stage, Status};

/// What a background thread reports when it ends.
pub(super) enum Event {
    /// A check ended with this status.
    Checked(Status),
    /// The install with this number ended.
    Installed(u64, Result<Outcome, Failure>),
    /// Opening a world with an older release ended.
    Opened(Result<(), Failure>),
}

/// The updater. Menus read it and ask it to check, install, cancel and restart; the network
/// work runs on background threads and never holds a frame.
///
/// Its default has no network: nothing is fetched until the game inserts one made with [`new`].
///
/// [`new`]: Updates::new
#[derive(Resource)]
pub struct Updates {
    /// Whether a newer release exists.
    pub(super) status: Status,
    /// How far the install has gone.
    pub(super) stage: Stage,
    /// How the newer release can be installed here.
    pub(super) plan: Plan,
    /// The network and the programs.
    pub(super) host: Arc<dyn Host>,
    /// Where the game runs.
    pub(super) place: Place,
    /// The version of the running game.
    pub(super) current: String,
    /// Where background threads report.
    pub(super) sender: Sender<Event>,
    /// What background threads reported and the game has not read yet.
    pub(super) inbox: Mutex<Receiver<Event>>,
    /// The cancel and progress signals of the install that runs, or ran last.
    pub(super) signals: Arc<Signals>,
    /// Number of the install that runs, or ran last. A report with another number is stale.
    pub(super) job: u64,
    /// How far opening a world with an older release has gone.
    pub(super) older: Older,
}

impl Default for Updates {
    fn default() -> Self {
        Self::new(
            Arc::new(Offline),
            Place::default(),
            env!("CARGO_PKG_VERSION"),
        )
    }
}

impl Updates {
    /// Returns an updater for the game of version `current` at `place`, working through `host`.
    pub fn new(host: Arc<dyn Host>, place: Place, current: &str) -> Self {
        let (sender, inbox) = mpsc::channel();
        Self {
            status: Status::Idle,
            stage: Stage::Idle,
            plan: Plan::default(),
            host,
            place,
            current: current.to_string(),
            sender,
            inbox: Mutex::new(inbox),
            signals: Arc::default(),
            job: 0,
            older: Older::Idle,
        }
    }

    /// Returns whether a newer release exists.
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Returns how far the install has gone.
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    /// Returns how the newer release can be installed on this computer.
    pub fn plan(&self) -> Plan {
        self.plan
    }

    /// Returns the version of the running game.
    pub fn current(&self) -> &str {
        &self.current
    }

    /// Returns the newer release, when one is known.
    pub fn release(&self) -> Option<&Release> {
        match &self.status {
            Status::Available(release) => Some(release),
            _ => None,
        }
    }

    /// Returns the bytes received and the size, in bytes, of the file the game installs from
    /// the newer release. Both are zero when there is none.
    pub fn progress(&self) -> (u64, u64) {
        let file = self.release().zip(self.place.platform);
        let size = file.and_then(|(release, platform)| pick(release, platform));
        (
            self.signals.done(),
            size.map_or(0, |(payload, _)| payload.size),
        )
    }
}
