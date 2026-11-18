//! The outside world as the updater sees it: fetching over the network and starting a program.
//!
//! The game uses [`super::net::Net`]. Tests pass their own, so no test touches the network.

use std::path::Path;

use super::failure::Failure;

/// What receives a download, one chunk at a time. An error stops the download.
pub type Sink<'a> = &'a mut dyn FnMut(&[u8]) -> Result<(), Failure>;

/// Fetches files and starts programs for the updater.
pub trait Host: Send + Sync + 'static {
    /// Returns the whole body at `url`, refusing one longer than `limit` bytes.
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>, Failure>;

    /// Hands the body at `url` to `sink` in chunks, in order.
    fn stream(&self, url: &str, sink: Sink) -> Result<(), Failure>;

    /// Starts `program` with `args` and does not wait for it.
    fn launch(&self, program: &Path, args: &[String]) -> Result<(), Failure>;
}

/// A host with no network and no programs: every call fails. It stands in where the updater
/// is not wired, so that nothing reaches the network by accident.
#[derive(Debug, Clone, Copy, Default)]
pub struct Offline;

impl Host for Offline {
    fn get(&self, _: &str, _: u64) -> Result<Vec<u8>, Failure> {
        Err(Failure::Offline)
    }

    fn stream(&self, _: &str, _: Sink) -> Result<(), Failure> {
        Err(Failure::Offline)
    }

    fn launch(&self, _: &Path, _: &[String]) -> Result<(), Failure> {
        Err(Failure::Offline)
    }
}
