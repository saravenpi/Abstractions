//! The updater inside the app: the main menu shows a newer release and runs a check, with a
//! host that never touches the network. The install is walked in the module below.

mod walk;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::input::InputPlugin;
use bevy::input_focus::{InputFocus, InputFocusPlugin};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::{activate, scratch, stubs};
use crate::menus::update::UpdateAction;
use crate::menus::update::home::{CheckLabel, OfferBox, OfferLine};
use crate::menus::{Menu, Nav};
use crate::settings::{Persistent, Settings};
use crate::ui::Selected;
use crate::update::check::LATEST;
use crate::update::fake::Fake;
use crate::update::fixture::document;
use crate::update::{Place, Platform, Stage, Status, Updates};
use crate::{computer, menus, screens, ui};

/// Builds the menu app around a game of version 0.9.0 for Linux, whose executable is `exe` in a
/// scratch folder and holds `old`, and whose updater works through `fake`. It starts on the
/// main menu.
fn start(tag: &str, fake: &Arc<Fake>, at_launch: bool, exe: &str) -> (App, PathBuf) {
    let dir = scratch(tag);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = std::fs::write(dir.join(exe), "old");
    let place = Place {
        platform: Some(Platform::Linux),
        exe: dir.join(exe),
        ..default()
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin));
    app.add_plugins((InputPlugin, InputFocusPlugin));
    stubs(&mut app, &dir);
    let mut settings = app.world_mut().resource_mut::<Persistent<Settings>>();
    let saved = settings.update(|settings| settings.updates.check_at_launch = at_launch);
    saved.unwrap();
    app.insert_resource(Updates::new(fake.clone(), place, "0.9.0"));
    app.add_plugins((screens::plugin, menus::plugin, ui::plugin));
    app.add_plugins(computer::plugin);
    crate::update::systems(&mut app);
    for _ in 0..3 {
        app.update();
    }
    (app, dir)
}

/// Runs frames, for a few seconds at most, until `done` holds, then one more to show it.
fn run_until(app: &mut App, done: impl Fn(&Updates) -> bool) {
    for _ in 0..5000 {
        app.update();
        if done(app.world().resource::<Updates>()) {
            app.update();
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("the updater never got there");
}

/// Runs frames until no check and no download runs.
fn settle(app: &mut App) {
    run_until(app, |updates| {
        *updates.status() != Status::Checking && *updates.stage() != Stage::Downloading
    });
}

/// Returns every text on screen.
fn texts(app: &mut App) -> Vec<String> {
    let mut query = app.world_mut().query::<&Text>();
    query.iter(app.world()).map(|text| text.0.clone()).collect()
}

/// Returns the text marked with `C` and whether the box around the newer release is shown.
fn home<C: Component>(app: &mut App) -> (String, bool) {
    let mut marked = app.world_mut().query_filtered::<&Text, With<C>>();
    let text = marked.single(app.world()).unwrap().0.clone();
    let mut boxes = app.world_mut().query_filtered::<&Node, With<OfferBox>>();
    let shown = boxes.single(app.world()).unwrap().display != Display::None;
    (text, shown)
}

/// With a newer release, the title shows the quiet line, the "Update" entry stands out without
/// taking the focus, and the running version stays in the corner.
#[test]
fn the_title_shows_the_newer_release_and_the_version() {
    let fake = Arc::new(Fake::with_release("v0.9.1", Platform::Linux, b"new"));
    let (mut app, _) = start("update-title", &fake, true, "abstractions");
    settle(&mut app);
    assert_eq!(fake.count(LATEST), 1);
    assert_eq!(
        home::<OfferLine>(&mut app),
        ("Version 0.9.1 is available".into(), true)
    );
    assert_eq!(home::<CheckLabel>(&mut app).0, "0.9.1 is available");
    let corner = format!("v{}", env!("CARGO_PKG_VERSION"));
    assert!(texts(&mut app).contains(&corner));
    let mut entries = app.world_mut().query::<(Entity, &Nav, Option<&Selected>)>();
    let update = entries
        .iter(app.world())
        .find(|(_, nav, _)| **nav == Nav::To(Menu::Update));
    let (update, _, chosen) = update.unwrap();
    assert_eq!(chosen, Some(&Selected(true)));
    let focused = app.world().resource::<InputFocus>().get().unwrap();
    assert_ne!(focused, update, "the entry does not steal the focus");
    assert_eq!(
        app.world().get::<Nav>(focused),
        Some(&Nav::To(Menu::Worlds))
    );
    assert_eq!(
        *app.world().resource::<State<Menu>>().get(),
        Menu::Main,
        "no popup"
    );
}

/// "Check for updates" runs a check: a waiting word, then the reason of a failure, then that
/// the game is up to date, then the newer version with its line.
#[test]
fn check_for_updates_runs_a_check() {
    let fake = Arc::new(Fake::default());
    let (mut app, _) = start("update-check", &fake, false, "abstractions");
    let label = home::<CheckLabel>;
    assert_eq!(label(&mut app), ("Check for updates".into(), false));
    fake.hold.store(true, Ordering::SeqCst);
    activate(&mut app, UpdateAction::Check);
    assert_eq!(label(&mut app), ("Checking...".into(), false));
    fake.hold.store(false, Ordering::SeqCst);
    settle(&mut app);
    assert_eq!(label(&mut app), ("No connection".into(), false));
    fake.serve(LATEST, &document("v0.9.0", &[]));
    activate(&mut app, UpdateAction::Check);
    settle(&mut app);
    assert_eq!(label(&mut app), ("You are up to date".into(), false));
    fake.serve(LATEST, &document("v0.10.0", &[]));
    activate(&mut app, UpdateAction::Check);
    settle(&mut app);
    assert_eq!(label(&mut app), ("0.10.0 is available".into(), true));
    assert_eq!(home::<OfferLine>(&mut app).0, "Version 0.10.0 is available");
    assert_eq!(fake.count(LATEST), 3);
    assert_eq!(fake.asked.lock().unwrap().len(), 3, "nothing is downloaded");
}

/// With the setting off nothing is checked at launch; with it on, exactly one check runs,
/// however long the title screen stays and whatever menus open.
#[test]
fn the_setting_decides_the_check_at_launch() {
    let fake = Arc::new(Fake::with_release("v0.9.1", Platform::Linux, b"new"));
    let (mut off, _) = start("update-off", &fake, false, "abstractions");
    for _ in 0..30 {
        off.update();
    }
    assert!(fake.asked.lock().unwrap().is_empty());
    assert_eq!(off.world().resource::<Updates>().status(), &Status::Idle);
    assert_eq!(home::<OfferLine>(&mut off), (String::new(), false));
    let (mut on, _) = start("update-on", &fake, true, "abstractions");
    settle(&mut on);
    activate(&mut on, Nav::To(Menu::Settings));
    activate(&mut on, Nav::Back);
    for _ in 0..30 {
        on.update();
    }
    assert_eq!(fake.asked.lock().unwrap().as_slice(), [LATEST]);
}
