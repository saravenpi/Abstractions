//! Opening and closing the computer, and the pointer while it is open.

use bevy::ecs::system::SystemParam;
use bevy::input::InputSystems;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use super::atlas::Atlas;
use super::view;
use crate::menus::Menu;
use crate::screens::Screen;
use crate::space::{Anchor, Location, Travel};
use crate::universe::Universe;

/// Key that opens and closes the computer.
pub const OPEN_KEY: KeyCode = KeyCode::Tab;

/// Whether the computer screen is open. The game keeps running under it.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Computer {
    /// The screen is hidden.
    #[default]
    Closed,
    /// The screen is shown.
    Open,
}

/// What decides whether the computer can be used.
#[derive(SystemParam)]
struct Seat<'w> {
    screen: Res<'w, State<Screen>>,
    menu: Res<'w, State<Menu>>,
    location: Option<Res<'w, Location>>,
}

impl Seat<'_> {
    /// Returns true while the player is seated in the ship with no menu open.
    fn taken(&self) -> bool {
        let playing = *self.screen.get() == Screen::Gameplay && *self.menu.get() == Menu::None;
        let seated = self.location.as_deref().map(|location| location.travel);
        playing && seated == Some(Travel::InShip)
    }
}

/// Opens the computer on its key while seated, and closes it on the same key, on Escape or
/// when the player leaves the seat. Escape is used up so that it does not also open the pause menu.
fn toggle(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    state: Res<State<Computer>>,
    mut next: ResMut<NextState<Computer>>,
    seat: Seat,
) {
    let open = *state.get() == Computer::Open;
    let escape =
        open && keys.just_pressed(KeyCode::Escape) && keys.clear_just_pressed(KeyCode::Escape);
    let pressed = escape || keys.just_pressed(OPEN_KEY);
    match (open, seat.taken()) {
        (true, false) => next.set(Computer::Closed),
        (true, true) if pressed => next.set(Computer::Closed),
        (false, true) if pressed => next.set(Computer::Open),
        _ => {}
    }
}

/// Releases and shows the pointer while the screen is open. Gameplay grabs it again afterwards.
fn free_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    if cursor.grab_mode != CursorGrabMode::None || !cursor.visible {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

/// Drops the keyboard focus held by a row of the closed screen.
fn leave(mut focus: ResMut<InputFocus>) {
    focus.clear();
}

/// Run condition: the screen is open and the world it reads from exists.
pub(super) fn ready(
    state: Res<State<Computer>>,
    universe: Option<Res<Universe>>,
    anchor: Option<Res<Anchor>>,
    location: Option<Res<Location>>,
    atlas: Option<Res<Atlas>>,
) -> bool {
    let world = universe.is_some() && anchor.is_some() && location.is_some() && atlas.is_some();
    *state.get() == Computer::Open && world
}

/// Registers the open state and its systems.
pub(super) fn plugin(app: &mut App) {
    app.init_state::<Computer>()
        .add_systems(PreUpdate, toggle.after(InputSystems))
        .add_systems(OnEnter(Computer::Open), view::spawn)
        .add_systems(OnExit(Computer::Open), leave)
        .add_systems(PostUpdate, free_cursor.run_if(in_state(Computer::Open)));
}
