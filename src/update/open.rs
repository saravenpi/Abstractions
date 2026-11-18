//! Starting things outside the game: the new version, the download page, the disk image.

use std::path::Path;

use super::failure::Failure;
use super::platform::opener;
use super::state::{Outcome, Stage};
use super::updates::Updates;

/// The page a player downloads the game from by hand.
pub const PAGE: &str = "https://abstractions.facile.studio";

impl Updates {
    /// Opens `target`, a file or an address, with the system's default application.
    fn open(&self, target: &str) -> Result<(), Failure> {
        let (program, lead) = opener(std::env::consts::OS);
        let mut args: Vec<String> = lead.iter().map(|arg| arg.to_string()).collect();
        args.push(target.to_string());
        self.host.launch(Path::new(program), &args)
    }

    /// Opens the download page of the game in the browser.
    pub fn open_page(&self) -> Result<(), Failure> {
        self.open(PAGE)
    }

    /// Opens the downloaded disk image, when the install left one.
    pub fn open_image(&self) -> Result<(), Failure> {
        match &self.stage {
            Stage::Ready(Outcome::Image(path)) => self.open(&path.to_string_lossy()),
            _ => Err(Failure::Unsupported),
        }
    }

    /// Starts the game again from its own path with the arguments it was started with. The
    /// caller then exits. Only an install that swapped the executable can be restarted into.
    pub fn restart(&self) -> Result<(), Failure> {
        match self.stage {
            Stage::Ready(Outcome::Swapped) => self.host.launch(&self.place.exe, &self.place.args),
            _ => Err(Failure::Unsupported),
        }
    }
}
