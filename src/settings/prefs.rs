//! Keeps the camera preferences of the rig and of the settings file the same.
//!
//! The rig reads and changes the [`ViewPrefs`] resource: the view key flips a mode and the wheel
//! zooms. The settings menu changes the copy inside [`Settings`]. Each side takes over what the
//! other changed, and a change made in play is written to the file once it has settled.

use bevy::prelude::*;

use super::{Persistent, Settings};
use crate::view::ViewPrefs;

/// Seconds a change made in play waits before it is written, so a turn of the wheel is one write.
pub(super) const SETTLE: f32 = 0.6;

/// What the keeper remembers between frames.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) struct Kept {
    /// The preferences of the settings as last seen, to tell a change of the menu from one in play.
    seen: Option<ViewPrefs>,
    /// The preferences of the rig on the last frame, to tell when they stopped changing.
    last: Option<ViewPrefs>,
    /// Seconds the rig has differed from the settings without changing.
    waited: f32,
}

/// Returns what the rig takes over from the settings: their preferences, when they changed since
/// they were last `seen`.
pub(super) fn adopted(seen: Option<ViewPrefs>, saved: ViewPrefs) -> Option<ViewPrefs> {
    (seen != Some(saved)).then_some(saved)
}

impl Kept {
    /// Returns the preferences of the rig `now` to write over the `saved` ones, `dt` seconds
    /// after the last frame: only once they differ and have not changed for [`SETTLE`]. A turn of
    /// the wheel, or the long glide of a trackpad, is then one write and not one every
    /// [`SETTLE`], and preferences that never equal themselves, as a NaN zoom, are never written.
    pub(super) fn due(&mut self, saved: ViewPrefs, now: ViewPrefs, dt: f32) -> Option<ViewPrefs> {
        let moved = self.last.replace(now) != Some(now);
        if saved == now || moved {
            self.waited = 0.0;
            return None;
        }
        self.waited += dt;
        if self.waited < SETTLE {
            return None;
        }
        (self.seen, self.waited) = (Some(now), 0.0);
        Some(now)
    }
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
    let Some(chosen) = kept.due(settings.view, *prefs, time.delta_secs()) else {
        return;
    };
    if let Err(error) = settings.update(|settings| settings.view = chosen) {
        warn!("could not save the view settings: {error}");
    }
}

/// Registers the keeper. The rig owns the resource; it is created here too so that the
/// settings run without the rig.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ViewPrefs>().add_systems(Update, keep);
}
