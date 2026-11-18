//! The pages of the update menu during and after an install: progress, restart, disk image.

use bevy::prelude::*;

use super::UpdateAction;
use super::format::{percent, progress_line};
use super::parts::{Fill, ProgressText, actions, bar, failure_line, para, title};
use crate::ui::Icon;
use crate::update::{Release, Stage, Updates};

/// What the restart page says under its title.
const RESTART_WHY: &str = "The new version is in place and runs the next time the game starts. \
The previous version is kept beside it until the new one has started.";

/// Why macOS gets a disk image, said again next to the Open button.
const IMAGE_WHY: &str = "The game does not rewrite its own application on macOS. The disk image \
is in your Downloads folder:";

/// The download page: the thin bar, percent and megabytes, and "Cancel".
pub fn downloading(page: &mut ChildSpawnerCommands, release: &Release) {
    page.spawn(title(format!("Downloading version {}", release.version)));
    page.spawn(bar());
    actions(
        page,
        Some((Some(Icon::Close), "Cancel", UpdateAction::Cancel)),
        false,
    );
}

/// The page after the swap: "Restart now" and "Later".
pub fn restart(page: &mut ChildSpawnerCommands, release: &Release) {
    page.spawn(title("Restart to finish"));
    page.spawn(para(format!(
        "Version {} is installed. {RESTART_WHY}",
        release.version
    )));
    page.spawn(failure_line());
    let main = (Some(Icon::Power), "Restart now", UpdateAction::Restart);
    actions(page, Some(main), true);
}

/// The macOS page after the download: what to do with the disk image named `name`, and "Open".
pub fn image(page: &mut ChildSpawnerCommands, release: &Release, name: &str) {
    page.spawn(title(
        "Open the disk image and drag Abstractions to Applications",
    ));
    page.spawn(para(format!(
        "Version {} is downloaded. {IMAGE_WHY} {name}",
        release.version
    )));
    page.spawn(failure_line());
    actions(page, Some((None, "Open", UpdateAction::OpenImage)), true);
}

/// Moves the progress bar and writes the line under it while a download runs.
pub(super) fn tick(
    updates: Res<Updates>,
    mut fills: Query<&mut Node, With<Fill>>,
    mut texts: Query<&mut Text, With<ProgressText>>,
) {
    if *updates.stage() != Stage::Downloading {
        return;
    }
    let (done, total) = updates.progress();
    let width = Val::Percent(percent(done, total) as f32);
    for mut node in fills.iter_mut().filter(|node| node.width != width) {
        node.width = width;
    }
    for mut text in &mut texts {
        text.set_if_neq(Text(progress_line(done, total)));
    }
}
