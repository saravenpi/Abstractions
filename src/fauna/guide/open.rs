//! Opening and closing the guide, and choosing a species in it.
//!
//! The guide key opens it on foot while the player controls the game and the map is closed. The
//! same key or Escape closes it, and so does leaving the ground, opening the map or a menu.
//! While it is open the arrow keys, the wheel and a click choose a species.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::window::PrimaryWindow;

use super::Guide;
use super::book::book;
use super::layout::{chars, measure};
use super::seen::Seen;
use super::view::{Slot, spawn};
use crate::audio::AudioCue;
use crate::computer::Computer;
use crate::space::{Location, Travel};
use crate::ui::Window;
use crate::universe::Universe;
use crate::world::Terrain;

/// What the open guide keeps: its list, the planet's name, the window of the list and the
/// measures it was drawn with.
#[derive(Resource, Debug, Clone, Default)]
pub struct Shelf {
    /// The list.
    pub book: super::book::Book,
    /// The name of the planet underfoot.
    pub planet: String,
    /// The selected row and the first drawn one.
    pub window: Window,
    /// Rows of the list drawn.
    pub rows: usize,
    /// Characters of small text that fit across the card.
    pub chars: usize,
}

/// Returns the state the guide moves to, if any: it opens on its key where it `can` be open,
/// and closes on its key, on Escape, or once it cannot be open.
pub fn next_state(open: bool, can: bool, (key, escape): (bool, bool)) -> Option<Guide> {
    match open {
        true if !can || key || escape => Some(Guide::Closed),
        false if can && key => Some(Guide::Open),
        _ => None,
    }
}

/// Opens and closes the guide. `playing` says the world is shown with no menu open.
pub(super) fn toggle(
    In(playing): In<bool>,
    (keys, settings): (
        Res<ButtonInput<KeyCode>>,
        Res<crate::settings::Persistent<crate::settings::Settings>>,
    ),
    (state, computer): (Res<State<Guide>>, Option<Res<State<Computer>>>),
    location: Option<Res<Location>>,
    (mut next, mut cues): (ResMut<NextState<Guide>>, MessageWriter<AudioCue>),
) {
    let map = computer.is_some_and(|c| *c.get() == Computer::Open);
    let foot = location.is_some_and(|l| l.travel == Travel::OnFoot);
    let open = *state.get() == Guide::Open;
    let pressed = (
        keys.just_pressed(settings.controls.keys.guide),
        keys.just_pressed(KeyCode::Escape),
    );
    let Some(wanted) = next_state(open, playing && foot && !map, pressed) else {
        return;
    };
    next.set(wanted);
    if pressed.0 {
        cues.write(match wanted {
            Guide::Open => AudioCue::UiConfirm,
            Guide::Closed => AudioCue::UiCancel,
        });
    }
}

/// Fills the shelf from the planet underfoot and spawns the guide.
pub(super) fn enter(
    (terrain, seen): (Option<Res<Terrain>>, Res<Seen>),
    (universe, location): (Option<Res<Universe>>, Option<Res<Location>>),
    window: Query<&bevy::window::Window, With<PrimaryWindow>>,
    mut shelf: ResMut<Shelf>,
    mut commands: Commands,
) {
    let size = window
        .single()
        .map_or(Vec2::new(1280.0, 720.0), |w| w.size());
    let measures = measure(size);
    let life = terrain.as_deref().and_then(|t| t.globe().life());
    let planet = universe.zip(location).map(|(u, l)| u.planet(l.body).name);
    *shelf = Shelf {
        book: book(life, &seen),
        planet: planet.unwrap_or_default(),
        window: Window::default(),
        rows: measures.rows,
        chars: chars(measures.card),
    };
    spawn(size, &mut commands);
}

/// Rebuilds the list when a species is seen while the guide is open.
pub(super) fn refresh(terrain: Option<Res<Terrain>>, seen: Res<Seen>, mut shelf: ResMut<Shelf>) {
    let life = terrain.as_deref().and_then(|t| t.globe().life());
    shelf.book = book(life, &seen);
}

/// Moves the selection with the arrow keys, Page Up and Page Down.
pub(super) fn step(keys: Res<ButtonInput<KeyCode>>, mut shelf: ResMut<Shelf>) {
    let page = shelf.rows.max(1) as isize;
    let moves = [
        (KeyCode::ArrowDown, 1),
        (KeyCode::ArrowUp, -1),
        (KeyCode::PageDown, page),
        (KeyCode::PageUp, -page),
    ];
    let by: isize = moves
        .iter()
        .filter(|(k, _)| keys.just_pressed(*k))
        .map(|(_, by)| by)
        .sum();
    if by != 0 {
        let (len, rows) = (shelf.book.entries.len(), shelf.rows);
        shelf.window.step(len, rows, by);
    }
}

/// Scrolls the list with the mouse wheel.
pub(super) fn wheel(wheel: Res<AccumulatedMouseScroll>, mut shelf: ResMut<Shelf>) {
    if wheel.delta.y == 0.0 {
        return;
    }
    let by = -wheel.delta.y.signum() as isize;
    let (len, rows) = (shelf.book.entries.len(), shelf.rows);
    shelf.window.scroll(len, rows, by);
}

/// Selects the row a click activates.
pub(super) fn pick(event: On<Activate>, slots: Query<&Slot>, mut shelf: ResMut<Shelf>) {
    let Ok(Slot(slot)) = slots.get(event.entity) else {
        return;
    };
    let len = shelf.book.entries.len();
    if let Some(index) = shelf.window.index(len, *slot) {
        shelf.window.cursor = index;
    }
}
