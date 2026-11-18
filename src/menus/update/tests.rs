//! Wording tests of the update entries and screens.

use std::path::PathBuf;

use super::format::{
    LABEL_CHARS, check_label, megabytes, offer_line, percent, progress_line, summary,
};
use super::view::View;
use crate::ui::theme;
use crate::update::release::parse;
use crate::update::{Failure, Outcome, Release, Stage, Status, Updates};

/// Returns a release tagged `v0.9.1`, published on a known day, without files.
fn release() -> Release {
    parse(br#"{"tag_name":"v0.9.1","published_at":"2026-11-20T09:30:00Z"}"#).unwrap()
}

/// Sizes show in megabytes with one decimal, and percent never passes 100.
#[test]
fn sizes_and_percent_read_plainly() {
    assert_eq!(megabytes(0), "0.0 MB");
    assert_eq!(megabytes(88_289_280), "84.2 MB");
    assert_eq!((percent(0, 0), percent(0, 10), percent(5, 10)), (0, 0, 50));
    assert_eq!(
        (percent(999, 1000), percent(10, 10), percent(30, 10)),
        (99, 100, 100)
    );
    assert_eq!(
        progress_line(37_014_733, 88_289_280),
        "41% - 35.3 of 84.2 MB"
    );
    assert_eq!(progress_line(0, 0), "0% - 0.0 of 0.0 MB");
    assert_eq!(
        progress_line(88_289_280, 88_289_280),
        "100% - checking the download"
    );
}

/// The check entry names every state, and every label fits the entry at the menu's width.
#[test]
fn the_check_entry_names_every_state() {
    assert_eq!(check_label(&Status::Idle), "Check for updates");
    assert_eq!(check_label(&Status::Checking), "Checking...");
    assert_eq!(check_label(&Status::UpToDate), "You are up to date");
    assert_eq!(
        check_label(&Status::Available(release())),
        "0.9.1 is available"
    );
    let reasons = [
        Failure::Offline,
        Failure::Timeout,
        Failure::Status(403),
        Failure::Unreadable,
    ];
    for failure in reasons {
        let reason = failure.reason();
        assert_eq!(
            check_label(&Status::Failed(reason.clone())),
            reason,
            "shown whole"
        );
    }
    let long = check_label(&Status::Failed("x".repeat(90)));
    assert_eq!(long.chars().count(), LABEL_CHARS);
    let glyph = theme::GLYPH_ASPECT * theme::FONT_BODY;
    let around = 2.0 * (theme::ICON + theme::GAP_S) + 2.0 * theme::GAP_S;
    assert!(LABEL_CHARS as f32 * glyph + around <= theme::MENU_WIDTH);
}

/// The line of the main menu follows the install, and the summary leaves out what is unknown.
#[test]
fn the_lines_follow_the_install() {
    let release = release();
    assert_eq!(
        offer_line(&release, &Stage::Idle),
        "Version 0.9.1 is available"
    );
    assert_eq!(
        offer_line(&release, &Stage::Failed("x".into())),
        "Version 0.9.1 is available"
    );
    assert_eq!(
        offer_line(&release, &Stage::Downloading),
        "Downloading version 0.9.1"
    );
    let swapped = Stage::Ready(Outcome::Swapped);
    assert_eq!(
        offer_line(&release, &swapped),
        "Version 0.9.1 is installed: restart to finish"
    );
    let image = Stage::Ready(Outcome::Image(PathBuf::from("a.dmg")));
    assert_eq!(
        offer_line(&release, &image),
        "Version 0.9.1 is in your Downloads folder"
    );
    let full = summary(&release, 88_289_280, "0.9.0");
    assert_eq!(full, "Released 2026-11-20 - 84.2 MB - you have 0.9.0");
    let bare = parse(br#"{"tag_name":"v0.9.1"}"#).unwrap();
    assert_eq!(summary(&bare, 0, "0.9.0"), "you have 0.9.0");
}

/// Without a newer release the update menu has nothing to offer.
#[test]
fn the_menu_without_a_release_offers_nothing() {
    assert_eq!(View::of(&Updates::default()), View::Nothing);
}
