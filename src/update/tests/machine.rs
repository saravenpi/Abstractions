//! Status tests: a check goes from idle to its result, and never runs twice at once.

use std::sync::Arc;

use super::{place, scratch, settle, updater};
use crate::update::check::LATEST;
use crate::update::fake::Fake;
use crate::update::job::{NO_BUILD, NO_FILE, READ_ONLY};
use crate::update::{PAGE, Place, Plan, Platform, Stage, Status, Updates};

/// A check goes from idle to checking to a newer release, with a plan to install it.
#[test]
fn a_check_goes_from_idle_to_available() {
    let (mut updates, fake, _) = updater("available", Platform::Linux, "v0.9.1");
    assert_eq!(
        (updates.status(), updates.stage()),
        (&Status::Idle, &Stage::Idle)
    );
    assert_eq!(updates.release(), None);
    updates.check();
    assert_eq!(updates.status(), &Status::Checking);
    updates.check();
    settle(&mut updates);
    assert_eq!(fake.count(LATEST), 1, "one check at a time");
    let release = updates.release().unwrap();
    assert_eq!(release.version.to_string(), "0.9.1");
    assert_eq!(updates.plan(), Plan::Swap);
    assert_eq!(updates.progress(), (0, 5000));
    assert_eq!(updates.current(), "0.9.0");
}

/// A check with no network, or with an unreadable answer, fails with a plain reason, and a
/// later check can succeed.
#[test]
fn a_failed_check_says_why_and_can_be_retried() {
    let dir = scratch("failed-check");
    let fake = Arc::new(Fake::default());
    let mut updates = Updates::new(fake.clone(), place(&dir, Platform::Linux), "0.9.0");
    updates.check();
    settle(&mut updates);
    assert_eq!(updates.status(), &Status::Failed("No connection".into()));
    fake.serve(LATEST, b"<html>rate limited</html>");
    updates.check();
    settle(&mut updates);
    assert_eq!(
        updates.status(),
        &Status::Failed("Unreadable answer".into())
    );
    let newer = Fake::with_release("v0.9.0", Platform::Linux, b"same");
    fake.serve(LATEST, &newer.files.lock().unwrap()[LATEST]);
    updates.check();
    settle(&mut updates);
    assert_eq!(updates.status(), &Status::UpToDate);
    assert_eq!(fake.count(LATEST), 3);
}

/// The default updater has no network: a check fails at once and nothing is fetched.
#[test]
fn the_default_updater_is_offline() {
    let mut updates = Updates::default();
    assert_eq!(updates.current(), env!("CARGO_PKG_VERSION"));
    updates.check();
    settle(&mut updates);
    assert_eq!(updates.status(), &Status::Failed("No connection".into()));
    updates.install();
    assert_eq!(updates.stage(), &Stage::Idle);
    assert!(updates.restart().is_err() && updates.open_image().is_err());
}

/// Without a build for this computer, without the plain file, or in a read-only folder, the
/// game offers the download page instead of installing.
#[test]
fn a_game_that_cannot_install_offers_the_page() {
    let (mut updates, fake, dir) = updater("page", Platform::Linux, "v0.9.1");
    updates.place.exe = dir.join("missing").join("abstractions");
    updates.check();
    settle(&mut updates);
    assert_eq!(updates.plan(), Plan::Page(READ_ONLY));
    updates.install();
    assert_eq!(updates.stage(), &Stage::Idle);
    assert_eq!(fake.asked.lock().unwrap().len(), 1, "only the check");
    updates.open_page().unwrap();
    let launched = fake.launched.lock().unwrap().clone();
    assert_eq!(launched[0].1.last().map(String::as_str), Some(PAGE));
    let other = Place {
        platform: None,
        ..place(&dir, Platform::Linux)
    };
    let release = updates.release().unwrap().clone();
    assert_eq!(
        crate::update::job::plan(&other, &release),
        Plan::Page(NO_BUILD)
    );
    let windows = place(&dir, Platform::Windows);
    assert_eq!(
        crate::update::job::plan(&windows, &release),
        Plan::Page(NO_FILE)
    );
}
