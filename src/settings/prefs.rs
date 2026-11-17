//! Keeps the camera preferences of the rig and of the settings file the same.
//!
//! The rig reads and changes the [`ViewPrefs`] resource: the view key flips a mode and the wheel
//! zooms. The settings menu changes the copy inside [`Settings`]. Each side takes over what the
//! other changed, and a change made in play is written to the file once it has settled.

use bevy::prelude::*;

use super::{Persistent, Settings};
use crate::view::ViewPrefs;

/// Seconds a change made in play waits before it is written, so a turn of the wheel is one write.
const SETTLE: f32 = 0.6;

/// What the keeper remembers between frames.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Kept {
    /// The preferences of the settings as last seen, to tell a change of the menu from one in play.
    seen: Option<ViewPrefs>,
    /// Seconds the rig has differed from the settings.
    waited: f32,
}

/// Returns what the rig takes over from the settings: their preferences, when they changed since
/// they were last `seen`.
pub(super) fn adopted(seen: Option<ViewPrefs>, saved: ViewPrefs) -> Option<ViewPrefs> {
    (seen != Some(saved)).then_some(saved)
}

/// Hands a change of the settings to the rig, and writes a change of the rig to the settings
/// once it has rested for [`SETTLE`].
fn keep(
    time: Res<Time>,
    mut kept: Local<Kept>,
    mut settings: ResMut<Persistent<Settings>>,
    mut prefs: ResMut<ViewPrefs>,
) {
    if let Some(saved) = adopted(kept.seen, settings.view) {
        kept.seen = Some(saved);
        prefs.set_if_neq(saved);
    }
    if settings.view == *prefs {
        kept.waited = 0.0;
        return;
    }
    kept.waited += time.delta_secs();
    if kept.waited < SETTLE {
        return;
    }
    let chosen = *prefs;
    (kept.seen, kept.waited) = (Some(chosen), 0.0);
    if let Err(error) = settings.update(|settings| settings.view = chosen) {
        warn!("could not save the view settings: {error}");
    }
}

/// Registers the keeper. The rig owns the resource; it is created here too so that the
/// settings run without the rig.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ViewPrefs>().add_systems(Update, keep);
}
