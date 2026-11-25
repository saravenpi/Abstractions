//! Opening and closing the pack screen.
//!
//! The pack key opens it on foot while the player controls the game and the map and the field
//! guide are closed. The same key or Escape closes it, and so does leaving the ground or opening a
//! menu. The Close button of its footer runs the same deed as Escape.

use bevy::prelude::*;

use super::Workshop;
use super::bench::{Bench, Pane};
use super::view::spawn;
use crate::audio::AudioCue;
use crate::computer::Computer;
use crate::fauna::guide::Guide;
use crate::space::{Location, Travel};

/// Returns the state the screen moves to, if any: it opens on its key where it `can` be open,
/// and closes on its key, on Escape, or once it cannot be open.
pub fn next_state(open: bool, can: bool, (key, escape): (bool, bool)) -> Option<Workshop> {
    match open {
        true if !can || key || escape => Some(Workshop::Closed),
        false if can && key => Some(Workshop::Open),
        _ => None,
    }
}

/// The screens that decide whether the pack opens: itself, the computer, the guide, and chests.
type Screens<'w> = (
    Res<'w, State<Workshop>>,
    Option<Res<'w, State<Computer>>>,
    Option<Res<'w, State<Guide>>>,
    Option<Res<'w, State<crate::craft::chest::ChestScreen>>>,
);

/// Opens and closes the screen. `playing` says the world is shown with no menu open.
pub(super) fn toggle(
    In(playing): In<bool>,
    (keys, settings): (
        Res<ButtonInput<KeyCode>>,
        Res<crate::settings::Persistent<crate::settings::Settings>>,
    ),
    (state, computer, guide, chest): Screens,
    location: Option<Res<Location>>,
    (mut next, mut cues): (ResMut<NextState<Workshop>>, MessageWriter<AudioCue>),
) {
    let map = computer.is_some_and(|c| *c.get() == Computer::Open);
    let foot = location.is_some_and(|l| l.travel == Travel::OnFoot);
    let open = *state.get() == Workshop::Open;
    let pressed = (
        keys.just_pressed(settings.controls.keys.pack),
        keys.just_pressed(KeyCode::Escape),
    );
    let can = playing
        && foot
        && !map
        && !crate::fauna::guide::shown(guide)
        && !crate::craft::chest::shown(chest);
    let Some(wanted) = next_state(open, can, pressed) else {
        return;
    };
    next.set(wanted);
    if pressed.0 {
        cues.write(match wanted {
            Workshop::Open => AudioCue::UiConfirm,
            Workshop::Closed => AudioCue::UiCancel,
        });
    }
}

/// Opens on the pack pane at its first row, keeps the intent and the focused card, and spawns
/// the screen.
pub(super) fn enter(mut bench: ResMut<Bench>, mut commands: Commands) {
    bench.pane = Pane::Pack;
    bench.cursor = 0;
    bench.made = None;
    bench.note.clear();
    spawn(&mut commands);
}
