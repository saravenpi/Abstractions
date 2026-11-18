//! A host for tests: it serves files from memory, records what it was asked, and starts nothing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::allow::RELEASES;
use super::check::LATEST;
use super::failure::Failure;
use super::fixture::{document, sha256};
use super::host::{Host, Sink};
use super::platform::Platform;

/// Bytes handed to the sink at a time.
const CHUNK: usize = 1024;

/// A host that never touches the network.
#[derive(Default)]
pub struct Fake {
    /// Body served at each address.
    pub files: Mutex<HashMap<String, Vec<u8>>>,
    /// Every address asked for, in order.
    pub asked: Mutex<Vec<String>>,
    /// Every program started, with its arguments.
    pub launched: Mutex<Vec<(PathBuf, Vec<String>)>>,
    /// While true, an answer waits before it returns and a download waits after each chunk.
    pub hold: AtomicBool,
}

impl Fake {
    /// Serves `body` at `url`.
    pub fn serve(&self, url: &str, body: &[u8]) {
        let mut files = self.files.lock().unwrap();
        files.insert(url.to_string(), body.to_vec());
    }

    /// Returns a host whose latest release is `tag`, with the file `payload` for `platform` and
    /// a checksum list that covers it.
    pub fn with_release(tag: &str, platform: Platform, payload: &[u8]) -> Self {
        let (name, sums) = (platform.payload(tag), platform.sums(tag));
        let list = format!("{}  {name}\n", sha256(payload));
        let files = [(name.clone(), payload.len()), (sums.clone(), list.len())];
        let fake = Self::default();
        fake.serve(LATEST, &document(tag, &files));
        fake.serve(&format!("{RELEASES}{tag}/{name}"), payload);
        fake.serve(&format!("{RELEASES}{tag}/{sums}"), list.as_bytes());
        fake
    }

    /// Records that `url` was asked for and returns what is served there.
    fn body(&self, url: &str) -> Result<Vec<u8>, Failure> {
        self.asked.lock().unwrap().push(url.to_string());
        let files = self.files.lock().unwrap();
        files.get(url).cloned().ok_or(Failure::Offline)
    }

    /// Waits while the host is held.
    fn wait(&self) {
        while self.hold.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Returns how many times `url` was asked for.
    pub fn count(&self, url: &str) -> usize {
        let asked = self.asked.lock().unwrap();
        asked.iter().filter(|seen| *seen == url).count()
    }
}

impl Host for Fake {
    fn get(&self, url: &str, _: u64) -> Result<Vec<u8>, Failure> {
        let body = self.body(url);
        self.wait();
        body
    }

    fn stream(&self, url: &str, sink: Sink) -> Result<(), Failure> {
        for chunk in self.body(url)?.chunks(CHUNK) {
            sink(chunk)?;
            self.wait();
        }
        Ok(())
    }

    fn launch(&self, program: &Path, args: &[String]) -> Result<(), Failure> {
        let mut launched = self.launched.lock().unwrap();
        launched.push((program.to_path_buf(), args.to_vec()));
        Ok(())
    }
}
