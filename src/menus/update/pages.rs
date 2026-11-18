//! The pages of the update menu shown before an install: the offer, and the two dead ends.

use bevy::prelude::*;

use super::UpdateAction;
use super::format::summary;
use super::parts::{NOTES_HEIGHT, actions, failure_line, para, title};
use super::steps;
use super::view::View;
use crate::ui::{Icon, scroller, text, theme};
use crate::update::{Plan, Release, Updates};

/// Why macOS gets a disk image instead of an install in place.
const IMAGE_WHY: &str = "On macOS the game does not rewrite its own application. It downloads \
the disk image to your Downloads folder, and you drag the new version to Applications.";

/// Spawns the release notes as plain lines in a column that scrolls.
fn notes(page: &mut ChildSpawnerCommands, release: &Release) {
    if release.notes.is_empty() {
        return;
    }
    page.spawn(scroller(NOTES_HEIGHT)).with_children(|column| {
        for line in &release.notes {
            column.spawn(text(line.clone(), theme::FONT_SMALL, theme::TEXT));
        }
    });
}

/// The offer: the version, its day and size, its notes, then "Download and install" and
/// "Later". When the game cannot install by itself it says why and offers the download page.
fn offer(page: &mut ChildSpawnerCommands, updates: &Updates, release: &Release) {
    let size = updates.progress().1;
    page.spawn(title(format!("Version {}", release.version)));
    page.spawn(para(summary(release, size, updates.current())));
    notes(page, release);
    let main = match updates.plan() {
        Plan::Swap => (
            Some(Icon::Check),
            "Download and install",
            UpdateAction::Install,
        ),
        Plan::Image => {
            page.spawn(para(IMAGE_WHY));
            (Some(Icon::Check), "Download", UpdateAction::Install)
        }
        Plan::Page(why) => {
            page.spawn(para(why));
            page.spawn(failure_line());
            (
                Some(Icon::Earth),
                "Open the download page",
                UpdateAction::OpenPage,
            )
        }
    };
    actions(page, Some(main), true);
}

/// The page shown when the menu is open without a newer release.
fn nothing(page: &mut ChildSpawnerCommands) {
    page.spawn(title("No update to install"));
    actions(page, None, true);
}

/// The page of a failed install: what failed, that the installed game is as it was, and a retry.
fn failed(page: &mut ChildSpawnerCommands, reason: &str) {
    page.spawn(title("The update failed"));
    page.spawn(para(format!(
        "{reason}. The installed game was not changed."
    )));
    actions(page, Some((None, "Try again", UpdateAction::Install)), true);
}

/// Spawns the page of `view` under the body of the menu.
pub fn fill(page: &mut ChildSpawnerCommands, view: &View, updates: &Updates) {
    match (view, updates.release()) {
        (View::Failed(reason), _) => failed(page, reason),
        (View::Offer, Some(release)) => offer(page, updates, release),
        (View::Downloading, Some(release)) => steps::downloading(page, release),
        (View::Restart, Some(release)) => steps::restart(page, release),
        (View::Image(name), Some(release)) => steps::image(page, release, name),
        _ => nothing(page),
    }
}
