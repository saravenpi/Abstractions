//! Opening a world named on the command line: `--world <id>`, which an older release is
//! started with to open its own world.

use bevy::prelude::*;

use super::{SaveDir, Starter, list};
use crate::screens::Screen;

/// Returns the id after `--world` in `args`, or in a `--world=<id>` argument. Nothing when the
/// flag is missing or has no id.
pub(super) fn world_arg(args: impl IntoIterator<Item = String>) -> Option<String> {
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--world" {
            return args.next().filter(|id| !id.is_empty());
        }
        if let Some(id) = arg.strip_prefix("--world=") {
            return Some(id).filter(|id| !id.is_empty()).map(str::to_string);
        }
    }
    None
}

/// Starts the world the command line names, once, at the first title screen. A world that does
/// not exist, or that an older release made, leaves the player at the title screen.
fn open(mut done: Local<bool>, dir: Res<SaveDir>, mut starter: Starter) {
    if std::mem::replace(&mut *done, true) {
        return;
    }
    let Some(id) = world_arg(std::env::args().skip(1)) else {
        return;
    };
    if let Some(entry) = list(&dir.0).into_iter().find(|entry| entry.id == id) {
        starter.start(entry);
    }
}

/// Registers the opening of the world named on the command line.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, open.run_if(in_state(Screen::Title)));
}
