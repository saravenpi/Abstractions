//! The input code of the body: the keyboard and the view of this machine as intents of the
//! local player. Only here, the camera and the interface may assume there is one player.

use bevy::prelude::*;

use super::intent::axis;
use super::lead::View;
use crate::settings::{Persistent, Settings};
use crate::sim::{Intent, IntentOf, PlayerId};
use crate::view::{Heading, Look, ViewPrefs};

/// Returns the intents the held keys `keys` and the `view` ask for under the keymap of `settings`.
fn intents(
    keys: &ButtonInput<KeyCode>,
    settings: &Settings,
    view: View,
    turn: f32,
) -> [Option<Intent>; 5] {
    let map = &settings.controls.keys;
    [
        Some(Intent::Move {
            forward: axis(keys, map.forward, map.back),
            strafe: axis(keys, map.right, map.left),
        }),
        Some(Intent::Look {
            yaw: view.yaw,
            pitch: view.pitch,
            turn,
            behind: view.third,
        }),
        keys.pressed(map.jump).then_some(Intent::Jump),
        keys.pressed(map.sprint).then_some(Intent::Boost),
        keys.just_pressed(map.wings).then_some(Intent::Wings),
    ]
}

/// Sends what the keys and the view ask this frame as intents of the local player.
///
/// The view is the one the camera rig will show: the angles it holds, turned by what the mouse
/// and the wing keys ask. It runs on every frame, so that a key held down since a menu or the
/// ship does not count as pressed again when the body takes over.
pub(super) fn send(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    (heading, look, prefs): (Res<Heading>, Res<Look>, Res<ViewPrefs>),
    mut out: MessageWriter<IntentOf>,
) {
    let view = View::ahead(&heading, &look, prefs.on_foot);
    let asked = intents(&keys, &settings, view, look.yaw_delta);
    out.write_batch(asked.into_iter().flatten().map(|intent| IntentOf {
        player: PlayerId::LOCAL,
        intent,
    }));
}
