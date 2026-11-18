//! The hand on the camera: the mouse, the wheel and the view key.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::io::{Kind, Look, Target};
use super::mode::ViewPrefs;
use super::tune::PIXELS_PER_NOTCH;
use crate::settings::{Persistent, Settings};

/// Returns the mouse movement the view takes: all of it, except in the ship while the button
/// that steers it is `held`, when the mouse turns the hull and the view stays where it is.
pub(super) fn free(motion: Vec2, kind: Kind, held: bool) -> Vec2 {
    match kind == Kind::Ship && held {
        true => Vec2::ZERO,
        false => motion,
    }
}

/// Reads the mouse and the wheel into [`Look`], with the mouse settings applied. In the ship
/// the mouse looks around only while the left button is up: held, it steers the hull.
pub(super) fn read_look(
    (motion, wheel): (Res<AccumulatedMouseMotion>, Res<AccumulatedMouseScroll>),
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    target: Res<Target>,
    settings: Res<Persistent<Settings>>,
    mut look: ResMut<Look>,
) {
    let held = buttons.is_some_and(|buttons| buttons.pressed(MouseButton::Left));
    let controls = &settings.controls;
    let notches = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y,
        MouseScrollUnit::Pixel => wheel.delta.y / PIXELS_PER_NOTCH,
    };
    *look = Look::from_mouse(
        free(motion.delta, target.kind, held),
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
