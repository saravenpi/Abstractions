//! The update menu walked inside the app: offer, download, progress, restart, and the ways out.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::{run_until, settle, start, texts};
use crate::menus::tests::{activate, tap};
use crate::menus::update::UpdateAction;
use crate::menus::update::parts::{Fill, Notice, ProgressText};
use crate::menus::{Menu, Nav};
use crate::update::allow::RELEASES;
use crate::update::fake::Fake;
use crate::update::{PAGE, Platform, Stage, Updates};

/// The bytes of a made-up new game: five chunks of the fake host.
const PAYLOAD: [u8; 5120] = [7; 5120];

/// Activates the only button that does `wanted`, without running a frame, so that what the
/// press sends can still be read.
fn press(app: &mut App, wanted: UpdateAction) {
    let mut buttons = app.world_mut().query::<(Entity, &UpdateAction)>();
    let mut found = buttons
        .iter(app.world())
        .filter(|(_, action)| **action == wanted);
    let (entity, _) = found.next().unwrap();
    assert!(found.next().is_none(), "one button does it");
    app.world_mut().trigger(Activate { entity });
}

/// Returns whether some text on screen contains `part`.
fn shows(app: &mut App, part: &str) -> bool {
    texts(app).iter().any(|text| text.contains(part))
}

/// Returns the width of the white part of the progress bar and the line under it.
fn bar(app: &mut App) -> (Val, String) {
    let mut bars = app.world_mut().query_filtered::<&Node, With<Fill>>();
    let width = bars.single(app.world()).unwrap().width;
    let mut lines = app
        .world_mut()
        .query_filtered::<&Text, With<ProgressText>>();
    (width, lines.single(app.world()).unwrap().0.clone())
}

/// Opens the update menu from the main menu once the check at launch has ended.
fn open(app: &mut App) {
    settle(app);
    activate(app, Nav::To(Menu::Update));
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Update);
}

/// What the offer of the fake release shows: its version, day, notes and both actions.
const OFFER: [&str; 7] = [
    "Version 0.9.1",
    "Released 2026-11-20",
    "you have 0.9.0",
    "What's Changed",
    "- Rivers run to the sea",
    "Download and install",
    "Later",
];

/// The offer shows the version, the day, the notes and two actions, focuses the install, and
/// downloads nothing.
#[test]
fn the_offer_shows_the_release_and_downloads_nothing() {
    let fake = Arc::new(Fake::with_release("v0.9.1", Platform::Linux, &PAYLOAD));
    let (mut app, dir) = start("update-offer", &fake, true, "abstractions");
    open(&mut app);
    for part in OFFER {
        assert!(shows(&mut app, part), "{part}");
    }
    let focused = app.world().resource::<InputFocus>().get().unwrap();
    let action = app.world().get::<UpdateAction>(focused);
    assert_eq!(action, Some(&UpdateAction::Install));
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(fake.asked.lock().unwrap().len(), 1, "only the check");
    assert!(!dir.join("abstractions.part").exists());
}

/// The download shows a bar, percent and "Cancel". Then the new game is in place, and
/// "Restart now" starts it with the old one's path and exits.
#[test]
fn the_update_screen_walks_an_install() {
    let fake = Arc::new(Fake::with_release("v0.9.1", Platform::Linux, &PAYLOAD));
    let (mut app, dir) = start("update-walk", &fake, true, "abstractions");
    open(&mut app);
    fake.hold.store(true, Ordering::SeqCst);
    activate(&mut app, UpdateAction::Install);
    run_until(&mut app, |updates| updates.progress().0 > 0);
    assert!(shows(&mut app, "Downloading version 0.9.1") && shows(&mut app, "Cancel"));
    let wanted = (Val::Percent(20.0), "20% - 0.0 of 0.0 MB".into());
    assert_eq!(bar(&mut app), wanted);
    fake.hold.store(false, Ordering::SeqCst);
    settle(&mut app);
    assert!(shows(&mut app, "Restart to finish") && shows(&mut app, "Later"));
    assert_eq!(std::fs::read(dir.join("abstractions")).unwrap(), PAYLOAD);
    let kept = std::fs::read_to_string(dir.join("abstractions.old"));
    assert_eq!(kept.unwrap(), "old");
    assert!(app.should_exit().is_none());
    press(&mut app, UpdateAction::Restart);
    let started = [(dir.join("abstractions"), Vec::new())];
    assert_eq!(*fake.launched.lock().unwrap(), started);
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}

/// A damaged download shows what failed and leaves the game as it was; a retry can be
/// cancelled, which returns to the offer and leaves no file behind.
#[test]
fn a_failure_and_a_cancel_leave_the_game_untouched() {
    let fake = Arc::new(Fake::with_release("v0.9.1", Platform::Linux, &PAYLOAD));
    let (mut app, dir) = start("update-fail", &fake, true, "abstractions");
    let file = format!("{RELEASES}v0.9.1/abstractions-v0.9.1-linux-x86_64");
    fake.serve(&file, &vec![8u8; 5120]);
    open(&mut app);
    activate(&mut app, UpdateAction::Install);
    settle(&mut app);
    assert!(shows(&mut app, "The update failed") && shows(&mut app, "Try again"));
    assert!(shows(
        &mut app,
        "checksum does not match. The installed game was not changed."
    ));
    fake.serve(&file, &PAYLOAD);
    fake.hold.store(true, Ordering::SeqCst);
    activate(&mut app, UpdateAction::Install);
    run_until(&mut app, |updates| updates.progress().0 > 0);
    assert!(dir.join("abstractions.part").exists());
    activate(&mut app, UpdateAction::Cancel);
    assert_eq!(app.world().resource::<Updates>().stage(), &Stage::Idle);
    assert!(shows(&mut app, "Download and install"));
    fake.hold.store(false, Ordering::SeqCst);
    let part = dir.join("abstractions.part");
    run_until(&mut app, |_| !part.exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("abstractions")).unwrap(),
        "old"
    );
    assert!(!dir.join("abstractions.old").exists());
    activate(&mut app, Nav::Back);
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Main);
}

/// A game in a folder it cannot write to explains why and offers the download page instead of
/// an install. Escape goes back to the main menu.
#[test]
fn a_read_only_game_offers_the_download_page() {
    let fake = Arc::new(Fake::with_release("v0.9.1", Platform::Linux, &PAYLOAD));
    let (mut app, _) = start("update-page", &fake, false, "missing/abstractions");
    activate(&mut app, UpdateAction::Check);
    open(&mut app);
    assert!(shows(&mut app, "cannot write to its own folder"));
    assert!(shows(&mut app, "Open the download page") && !shows(&mut app, "Download and install"));
    activate(&mut app, UpdateAction::OpenPage);
    let launched = fake.launched.lock().unwrap().clone();
    assert_eq!(launched[0].1.last().map(String::as_str), Some(PAGE));
    let mut notices = app.world_mut().query_filtered::<&Text, With<Notice>>();
    assert_eq!(
        notices.single(app.world()).unwrap().0,
        "",
        "the page opened"
    );
    assert_eq!(fake.asked.lock().unwrap().len(), 1, "nothing is downloaded");
    tap(&mut app, KeyCode::Escape);
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Main);
}
