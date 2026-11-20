//! The Go key and the home key in the ship with the map closed: Go sends the ship to the course
//! or stops the route it flies, home flies it to the home planet, without opening the map.

use bevy::prelude::*;

use crate::computer::{Go, HOME_KEY};
use crate::settings::{Persistent, Settings};

/// Presses Go on the engage key and flies home on the home key. A player on foot gets nothing:
/// the course line of the HUD says to board the ship first.
pub(super) fn go(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    mut go: Go,
) {
    if keys.just_pressed(settings.controls.keys.engage) {
        go.press(None);
    }
    if keys.just_pressed(HOME_KEY) {
        go.home();
    }
}
