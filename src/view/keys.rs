//! The hand on the camera: the mouse, the wheel and the view key.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::io::{Look, Target};
use super::mode::ViewPrefs;
use super::tune::PIXELS_PER_NOTCH;
use crate::settings::{Persistent, Settings};

/// Reads the mouse and the wheel into [`Look`], with the mouse settings applied.
pub(super) fn read_look(
    motion: Res<AccumulatedMouseMotion>,
    wheel: Res<AccumulatedMouseScroll>,
    settings: Res<Persistent<Settings>>,
    mut look: ResMut<Look>,
) {
    let controls = &settings.controls;
    let notches = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y,
        MouseScrollUnit::Pixel => wheel.delta.y / PIXELS_PER_NOTCH,
    };
    *look = Look::from_mouse(
        motion.delta,
        notches,
        controls.mouse_sensitivity,
        controls.invert_y,
    );
}

/// Switches between first and third person for what the camera follows now, on foot or in
/// the ship, when the view key is pressed. The other situation keeps its own choice.
pub(super) fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<Persistent<Settings>>,
    target: Res<Target>,
    mut prefs: ResMut<ViewPrefs>,
) {
    if keys.just_pressed(settings.controls.keys.view) {
        prefs.toggle(target.kind);
    }
}
