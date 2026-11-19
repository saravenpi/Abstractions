//! The input code of the body: the keyboard and the view of this machine as intents of the
//! local player. Only here, the camera and the interface may assume there is one player.

use bevy::prelude::*;

use super::eyes::WingAim;
use super::intent::axis;
use super::lead::View;
use super::{Player, local};
use crate::gear::Gear;
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

/// The gear of every player body, among which the local one is found.
type Bodies<'w, 's> = Query<'w, 's, (&'static PlayerId, &'static Gear), With<Player>>;

/// Sends what the keys and the view ask this frame as intents of the local player.
///
/// The look is the view the camera rig will show: the angles it holds, turned by what the mouse
/// and the wing keys ask. On the wings it is the aim of the flight instead, which the view only
/// leads while the steer button is held. It runs on every frame, so that a key held down since
/// a menu or the ship does not count as pressed again when the body takes over.
pub(super) fn send(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    (heading, look, prefs): (Res<Heading>, Res<Look>, Res<ViewPrefs>),
    (wing, players): (Res<WingAim>, Bodies),
    mut out: MessageWriter<IntentOf>,
) {
    let seen = View::ahead(&heading, &look, prefs.on_foot);
    let gear = local(players.iter().map(|(id, gear)| (*id, gear)));
    let view = match gear.is_some_and(|gear| gear.state.mode.winged()) {
        true => View {
            yaw: wing.stick.aim.x,
            pitch: wing.stick.aim.y,
            ..seen
        },
        false => seen,
    };
    let asked = intents(&keys, &settings, view, look.yaw_delta);
    out.write_batch(asked.into_iter().flatten().map(|intent| IntentOf {
        player: PlayerId::LOCAL,
        intent,
    }));
}
