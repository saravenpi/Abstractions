//! The room with saving around it: the world goes to disk every 30 seconds, when the last
//! player leaves, and once more at shutdown, and joins and leaves are logged.
//!
//! Writing happens on a saver thread, so a slow disk never holds a tick back.

mod host;
mod roster;
mod saver;

#[cfg(test)]
mod roster_tests;

use std::io;
use std::path::PathBuf;
use std::time::Instant;

use abstractions_net::room::Room;
use abstractions_net::store;
use roster::Roster;
use saver::Saver;

/// Seconds between two saves while players are on.
const SAVE_EVERY: f64 = 30.0;

/// A [`Room`] that saves its world. It hands every network call to the room.
#[derive(Debug)]
pub struct Saving {
    /// The server's logic.
    room: Room,
    /// Data folder the world file sits in.
    dir: PathBuf,
    /// Network time of the next save.
    next: f64,
    /// The players last seen, to log who joins and leaves.
    roster: Roster,
    /// The thread that writes the world.
    saver: Saver,
    /// When the network clock started, near enough, for the last save.
    start: Instant,
}

impl Saving {
    /// Returns the room of `cap` seats, saving into `dir`. The room's network time must start
    /// now, at zero.
    pub fn new(room: Room, dir: PathBuf, cap: u16) -> io::Result<Self> {
        Ok(Self {
            room,
            saver: Saver::spawn(dir.clone())?,
            dir,
            next: SAVE_EVERY,
            roster: Roster::new(cap),
            start: Instant::now(),
        })
    }

    /// Logs who joined or left since the last look, and saves when the last player left.
    fn check(&mut self, now: f64) {
        let lines = self.roster.update(self.room.players());
        for line in &lines {
            println!("{line}");
        }
        if !lines.is_empty() && self.roster.is_empty() {
            self.save(now);
        }
    }

    /// Saves when the last save is [`SAVE_EVERY`] old and something changed since, so an idle
    /// server copies and writes nothing.
    fn due(&mut self, now: f64) {
        if now >= self.next {
            match self.room.changed() {
                true => self.save(now),
                false => self.next = now + SAVE_EVERY,
            }
        }
    }

    /// Hands the world at `now` to the saver thread and plans the next save.
    fn save(&mut self, now: f64) {
        if self.saver.offer(self.room.snapshot(now)) {
            self.room.saved();
        }
        self.next = now + SAVE_EVERY;
    }

    /// Waits for the saver thread, then writes the world one last time, here.
    pub fn finish(self) -> io::Result<()> {
        self.saver.finish();
        store::save(
            &self.dir,
            &self.room.snapshot(self.start.elapsed().as_secs_f64()),
        )
    }
}
