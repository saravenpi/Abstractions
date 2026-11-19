//! The input code of the body: the keyboard and the view of this machine as intents of the
//! local player. Only here, the camera and the interface may assume there is one player.

use bevy::prelude::*;

use super::eyes::STEER;
use super::intent::axis;
use super::lead::View;
use crate::settings::{Persistent, Settings};
use crate::sim::{Intent, IntentOf, PlayerId};
use crate::view::{Heading, Look, ViewPrefs};

/// Returns the intents the held keys `keys`, the steer button and the `view` ask for under the
/// keymap of `settings`. The yaw keys are the ship's turn keys.
fn intents(
    (keys, steer): (&ButtonInput<KeyCode>, bool),
    settings: &Settings,
    view: View,
    turn: f32,
) -> [Option<Intent>; 7] {
    let map = &settings.controls.keys;
    let yaw = axis(keys, map.ship.turn_right, map.ship.turn_left);
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
        (yaw != 0.0).then_some(Intent::Yaw(yaw)),
        steer.then_some(Intent::Steer),
    ]
}

/// Sends what the keys and the view ask this frame as intents of the local player.
///
/// The look is the view the camera rig will show: the angles it holds, turned by what the mouse
/// asks. On the wings it aims the body only while the steer button is held. It runs on every
/// frame, so that a key held down since a menu or the ship does not count as pressed again when
/// the body takes over.
pub(super) fn send(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    buttons: Res<ButtonInput<MouseButton>>,
    (heading, look, prefs): (Res<Heading>, Res<Look>, Res<ViewPrefs>),
    mut out: MessageWriter<IntentOf>,
) {
    let view = View::ahead(&heading, &look, prefs.on_foot);
    let held = (&*keys, buttons.pressed(STEER));
    let asked = intents(held, &settings, view, look.yaw_delta);
    out.write_batch(asked.into_iter().flatten().map(|intent| IntentOf {
        player: PlayerId::LOCAL,
        intent,
    }));
}
