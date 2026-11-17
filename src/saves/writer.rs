//! Writing world files off the main thread: one write at a time, the newest snapshot next.

use std::io;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::tasks::futures_lite::future::poll_once;
use bevy::tasks::{IoTaskPool, Task, block_on};

use super::autosave::{FLASH_SECONDS, SavedFlash};
use super::store::write;
use super::{ActiveWorld, SaveDir};

/// A write that failed, with the id of its world.
type Failure = (String, io::Error);

/// The save being written and the one waiting behind it.
///
/// Two writes of one world never run at once, so they cannot interleave on its temporary file.
/// A newer snapshot replaces the one that waits: only the latest state is worth writing.
#[derive(Resource, Default)]
pub struct Writer {
    /// The write in flight.
    task: Option<Task<Result<(), Failure>>>,
    /// The snapshot to write once the write in flight is done.
    next: Option<ActiveWorld>,
}

/// Writes `world` into `dir`. A failure leaves the file of the last good save as it was.
fn store(dir: &Path, world: &ActiveWorld) -> Result<(), Failure> {
    write(dir, &world.0.id, &world.0.save).map_err(|error| (world.0.id.clone(), error))
}

impl Writer {
    /// Queues `world` to be written, in place of a snapshot that still waits.
    pub fn queue(&mut self, world: ActiveWorld) {
        self.next = Some(world);
    }

    /// Returns true while a write is in flight or a snapshot waits.
    pub fn busy(&self) -> bool {
        self.task.is_some() || self.next.is_some()
    }

    /// Takes the result of the write in flight once it is done, without waiting for it.
    fn reap(&mut self) -> Option<Result<(), Failure>> {
        let done = block_on(poll_once(self.task.as_mut()?))?;
        self.task = None;
        Some(done)
    }

    /// Starts writing the waiting snapshot into `dir` when no write is in flight.
    fn start(&mut self, dir: PathBuf) {
        if self.task.is_some() {
            return;
        }
        if let Some(world) = self.next.take() {
            let job = async move { store(&dir, &world) };
            self.task = Some(IoTaskPool::get().spawn(job));
        }
    }

    /// Waits for the write in flight and writes the waiting snapshot before returning.
    ///
    /// This blocks: it is for the last frame of the application only.
    pub fn finish(&mut self, dir: &Path) {
        let flown = self.task.take().map(block_on);
        let last = self.next.take().map(|world| store(dir, &world));
        for (id, error) in [flown, last].into_iter().flatten().filter_map(Result::err) {
            warn!("could not save world {id}: {error}");
        }
    }
}

/// Collects the finished write, shows the "Saved" notice or warns, and starts the next write.
fn pump(dir: Res<SaveDir>, mut writer: ResMut<Writer>, mut flash: ResMut<SavedFlash>) {
    match writer.reap() {
        Some(Ok(())) => flash.0 = FLASH_SECONDS,
        Some(Err((id, error))) => warn!("could not save world {id}: {error}"),
        None => {}
    }
    writer.start(dir.0.clone());
}

/// Registers the writer, which only runs while it has something to do.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Writer>()
        .add_systems(Update, pump.run_if(|writer: Res<Writer>| writer.busy()));
}
