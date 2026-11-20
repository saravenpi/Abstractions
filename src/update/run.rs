//! What the updater does when asked: check, install, cancel, and take in what the threads report.

use std::sync::Arc;
use std::thread;

use super::check::check;
use super::failure::Failure;
use super::job;
use super::older::Older;
use super::signals::Signals;
use super::state::{Plan, Stage, Status};
use super::updates::{Event, Updates};

/// Runs `work` on a named background thread. Returns false when the thread could not start.
pub(super) fn background(name: &str, work: impl FnOnce() + Send + 'static) -> bool {
    let builder = thread::Builder::new().name(name.to_string());
    builder.spawn(work).is_ok()
}

impl Updates {
    /// Returns whether an install runs or is done: the release must not change under it.
    fn busy(&self) -> bool {
        matches!(self.stage, Stage::Downloading | Stage::Ready(_))
    }

    /// Looks for a newer release in the background. Does nothing while a check or an install runs.
    pub fn check(&mut self) {
        if self.status == Status::Checking || self.busy() {
            return;
        }
        let (host, sender, current) =
            (self.host.clone(), self.sender.clone(), self.current.clone());
        let started = background("update-check", move || {
            let _ = sender.send(Event::Checked(check(&*host, &current)));
        });
        self.status = match started {
            true => Status::Checking,
            false => Status::Failed(Failure::Disk("no thread".into()).reason()),
        };
    }

    /// Downloads and installs the newer release in the background. Does nothing without a newer
    /// release, while an install runs or is done, or when the game cannot install by itself.
    pub fn install(&mut self) {
        let Some(release) = self.release().cloned() else {
            return;
        };
        if self.busy() || matches!(self.plan, Plan::Page(_)) {
            return;
        }
        self.job += 1;
        self.signals = Arc::new(Signals::default());
        let (host, place, sender) = (self.host.clone(), self.place.clone(), self.sender.clone());
        let (signals, number) = (self.signals.clone(), self.job);
        let started = background("update-install", move || {
            let done = job::install(&*host, &place, &release, &signals);
            let _ = sender.send(Event::Installed(number, done));
        });
        self.stage = match started {
            true => Stage::Downloading,
            false => Stage::Failed(Failure::Disk("no thread".into()).reason()),
        };
    }

    /// Cancels the running install, unless it is already putting the new game in place. The
    /// thread removes its download.
    pub fn cancel(&mut self) {
        if self.stage == Stage::Downloading && self.signals.cancel() {
            self.stage = Stage::Idle;
        }
    }

    /// Takes in one report of a background thread. A report of a cancelled or older install is
    /// dropped.
    fn apply(&mut self, event: Event) {
        match event {
            Event::Checked(status) if self.status == Status::Checking => {
                if let Status::Available(release) = &status {
                    self.plan = job::plan(&self.place, release);
                }
                self.status = status;
            }
            Event::Installed(number, done) if self.installing(number) => {
                self.stage = match done {
                    Ok(outcome) => Stage::Ready(outcome),
                    Err(Failure::Cancelled) => Stage::Idle,
                    Err(failure) => Stage::Failed(failure.reason()),
                };
            }
            Event::Opened(done) if self.older == Older::Working => self.opened(done),
            _ => {}
        }
    }

    /// Returns whether install `number` is the one that runs.
    fn installing(&self, number: u64) -> bool {
        number == self.job && self.stage == Stage::Downloading
    }

    /// Takes in what the background threads reported. Returns whether anything arrived.
    pub fn poll(&mut self) -> bool {
        let inbox = self
            .inbox
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let events: Vec<Event> = inbox.try_iter().collect();
        let arrived = !events.is_empty();
        for event in events {
            self.apply(event);
        }
        arrived
    }
}
