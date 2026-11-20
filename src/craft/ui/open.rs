//! Opening and closing the pack screen, and scrolling its list.
//!
//! The pack key opens it on foot while the player controls the game and the map and the field
//! guide are closed. The same key or Escape closes it, unless a search is being typed, and so does
//! leaving the ground or opening a menu.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::Workshop;
use super::bench::Bench;
use super::here::Here;
use super::layout::fit;
use super::paint::arranged;
use super::view::spawn;
use crate::audio::AudioCue;
use crate::computer::Computer;
use crate::craft::pack::Pack;
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

/// Opens and closes the screen. `playing` says the world is shown with no menu open.
pub(super) fn toggle(
    In(playing): In<bool>,
    (keys, settings): (
        Res<ButtonInput<KeyCode>>,
        Res<crate::settings::Persistent<crate::settings::Settings>>,
    ),
    (state, computer, guide): (
        Res<State<Workshop>>,
        Option<Res<State<Computer>>>,
        Option<Res<State<Guide>>>,
    ),
    (location, bench): (Option<Res<Location>>, Res<Bench>),
    (mut next, mut cues): (ResMut<NextState<Workshop>>, MessageWriter<AudioCue>),
) {
    let map = computer.is_some_and(|c| *c.get() == Computer::Open);
    let foot = location.is_some_and(|l| l.travel == Travel::OnFoot);
    let open = *state.get() == Workshop::Open;
    let free = !bench.typing;
    let pressed = (
        free && keys.just_pressed(settings.controls.keys.pack),
        free && keys.just_pressed(KeyCode::Escape),
    );
    let can = playing && foot && !map && !crate::fauna::guide::shown(guide);
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

/// Measures the window, keeps the bench's choices but starts at the top, and spawns the screen.
pub(super) fn enter(
    window: Query<&bevy::window::Window, With<PrimaryWindow>>,
    mut bench: ResMut<Bench>,
    mut commands: Commands,
) {
    let size = window
        .single()
        .map_or(Vec2::new(1280.0, 720.0), |w| w.size());
    let (rows, _, chars) = fit(size);
    bench.rows = rows;
    bench.chars = chars;
    bench.typing = false;
    bench.window = Default::default();
    bench.note.clear();
    spawn(&mut commands);
}

/// Scrolls the list with the mouse wheel.
pub(super) fn wheel(
    wheel: Res<AccumulatedMouseScroll>,
    (pack, here): (Res<Pack>, Here),
    mut bench: ResMut<Bench>,
) {
    if wheel.delta.y == 0.0 {
        return;
    }
    let by = -wheel.delta.y.signum() as isize;
    let len = arranged(&bench, &pack, &here).len();
    let rows = bench.rows;
    bench.window.scroll(len, rows, by);
}
