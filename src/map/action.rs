//! The Go key in the ship with the map closed: Go sends the ship to the course or stops the
//! route it flies, without opening the map. A course to the home planet flies home through the
//! orbital cruise.

use bevy::prelude::*;

use crate::computer::Go;
use crate::settings::{Persistent, Settings};

/// Presses Go on the engage key. A player on foot gets nothing: the course line of the HUD says
/// to board the ship first.
pub(super) fn go(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    mut go: Go,
) {
    if keys.just_pressed(settings.controls.keys.engage) {
        go.press(None);
    }
}
