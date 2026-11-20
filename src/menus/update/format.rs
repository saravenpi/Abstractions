//! The words of the update entries and screens. Pure text, so it is tested without an app.

use crate::update::{Outcome, Release, Stage, Status};

/// Bytes in the megabyte the sizes are shown in.
const MEGABYTE: f64 = 1024.0 * 1024.0;

/// Most characters the label of a main menu entry holds without being cut.
pub const LABEL_CHARS: usize = 18;

/// Formats a size in megabytes with one decimal: `84.2 MB`.
pub fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / MEGABYTE)
}

/// Returns how much of `total` is `done`, in whole percent from 0 to 100.
pub fn percent(done: u64, total: u64) -> u32 {
    match total {
        0 => 0,
        _ => (done.min(total) as f64 * 100.0 / total as f64).floor() as u32,
    }
}

/// Returns the line under the progress bar: percent and megabytes, then the checking step once
/// every byte has arrived.
pub fn progress_line(done: u64, total: u64) -> String {
    if total > 0 && done >= total {
        return "100% - checking the download".to_string();
    }
    let (now, all) = (megabytes(done), megabytes(total));
    format!(
        "{}% - {} of {all}",
        percent(done, total),
        now.trim_end_matches(" MB")
    )
}

/// Returns the label of the "Check for updates" entry: its name, a waiting word while a check
/// runs, then the result. A long failure reason is cut to fit the entry.
pub fn check_label(status: &Status) -> String {
    let label = match status {
        Status::Idle => "Check for updates".to_string(),
        Status::Checking => "Checking...".to_string(),
        Status::UpToDate => "You are up to date".to_string(),
        Status::Available(release) => format!("{} available", release.version),
        Status::Failed(reason) => reason.clone(),
    };
    label.chars().take(LABEL_CHARS).collect()
}

/// Returns the quiet line of the main menu about `release`: that it is available, or how far
/// its install has gone.
pub fn offer_line(release: &Release, stage: &Stage) -> String {
    let version = release.version;
    match stage {
        Stage::Downloading => format!("Downloading version {version}"),
        Stage::Ready(Outcome::Swapped) => {
            format!("Version {version} is installed: restart to finish")
        }
        Stage::Ready(Outcome::Image(_)) => format!("Version {version} is in your Downloads folder"),
        Stage::Idle | Stage::Failed(_) => format!("Version {version} is available"),
    }
}

/// Returns the line under the version on the offer: the day of the release, the size of the
/// download when it is known, and the version that runs.
pub fn summary(release: &Release, size: u64, current: &str) -> String {
    let mut parts = Vec::new();
    if !release.date.is_empty() {
        parts.push(format!("Released {}", release.date));
    }
    if size > 0 {
        parts.push(megabytes(size));
    }
    parts.push(format!("you have {current}"));
    parts.join(" - ")
}
