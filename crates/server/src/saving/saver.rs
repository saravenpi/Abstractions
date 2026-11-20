//! The saver thread: it writes the worlds it is handed, one at a time.

use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

use abstractions_net::store::{self, World};

/// A thread that writes worlds to the data folder.
#[derive(Debug)]
pub struct Saver {
    /// Where worlds to write are sent. It holds one, so a slow disk skips saves.
    worlds: SyncSender<World>,
    /// The thread, which ends once the sender is dropped.
    thread: JoinHandle<()>,
}

impl Saver {
    /// Starts the thread that writes into `dir`.
    pub fn spawn(dir: PathBuf) -> io::Result<Self> {
        let (worlds, inbox) = sync_channel::<World>(1);
        let thread = thread::Builder::new()
            .name("abstractions-saver".to_string())
            .spawn(move || {
                for world in inbox {
                    if let Err(error) = store::save(&dir, &world) {
                        eprintln!("Could not save the world in {}: {error}", dir.display());
                    }
                }
            })?;
        Ok(Self { worlds, thread })
    }

    /// Hands `world` to the thread without waiting and returns true when it took it. It is
    /// dropped while a write is queued.
    pub fn offer(&self, world: World) -> bool {
        self.worlds.try_send(world).is_ok()
    }

    /// Lets the thread write what it holds and waits for it to end.
    pub fn finish(self) {
        drop(self.worlds);
        let _ = self.thread.join();
    }
}
