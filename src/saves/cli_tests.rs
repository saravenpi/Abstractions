//! Tests of the command line world and of the refusal to start an old world.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::cli::world_arg;
use super::model::WorldEntry;
use super::{ActiveWorld, Starter, WorldSave};
use crate::menus::Menu;
use crate::screens::Screen;

/// Returns `words` as the arguments of a program.
fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

/// The id after `--world` is read, in both spellings, and nothing else is.
#[test]
fn the_world_flag_gives_an_id() {
    assert_eq!(world_arg(args(&["--world", "reef"])), Some("reef".into()));
    assert_eq!(
        world_arg(args(&["-x", "--world=isle-2"])),
        Some("isle-2".into())
    );
    assert_eq!(world_arg(args(&["--world"])), None);
    assert_eq!(world_arg(args(&["--world="])), None);
    assert_eq!(world_arg(args(&["--seed", "7"])), None);
}

/// Starts `entry` in a bare app and returns whether a world became active.
fn starts(entry: WorldEntry) -> bool {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .init_state::<Screen>()
        .init_state::<Menu>();
    let _ = app
        .world_mut()
        .run_system_once(move |mut starter: Starter| {
            starter.start(entry.clone());
        });
    app.world().get_resource::<ActiveWorld>().is_some()
}

/// A world of version 2 is never started, and a world of this version is.
#[test]
fn an_old_world_is_never_started() {
    let entry = |version| WorldEntry {
        id: "reef".into(),
        save: WorldSave {
            version,
            ..WorldSave::default()
        },
    };
    assert!(!starts(entry(2)));
    assert!(starts(entry(super::SAVE_VERSION)));
}
