//! Tests of the spear in hand: a click stabs with it and never throws it, and only the throw key
//! draws and throws it.

use bevy::prelude::*;

use super::act::{Act, Ask, Timing, next};
use super::dir::Dir;
use super::path::Path;
use super::stage::spear;

/// Seconds in a frame at sixty frames per second.
const FRAME: f32 = 1.0 / 60.0;

/// A click with a spear in hand stabs and never draws it, however long the button is held: only the
/// throw key throws it, and the throw key does nothing to a tool not made to be thrown.
#[test]
fn a_click_stabs_with_a_spear() {
    let held = spear();
    let timing = Timing::of(&held, &Path::of(held.style, Dir::Right));
    let click = Ask {
        strike: true,
        ..default()
    };
    let (act, loosed) = next(Act::Ready, &click, &timing, true, FRAME);
    assert_eq!((act, loosed), (Act::Windup(0.0), None));
    let (mut act, mut loosed) = (act, None);
    for _ in 0..120 {
        let (after, let_go) = next(act, &click, &timing, true, FRAME);
        (act, loosed) = (after, loosed.or(let_go));
    }
    assert_eq!(loosed, None);
    let throw = Ask {
        throw: true,
        holding: true,
        ..default()
    };
    assert_eq!(
        next(Act::Ready, &throw, &timing, false, FRAME).0,
        Act::Ready
    );
}
