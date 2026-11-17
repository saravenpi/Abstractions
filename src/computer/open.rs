//! Opening and closing the computer.
//!
//! While the screen is open the keyboard and the mouse belong to it. The ship and the player
//! look at [`Computer`], or at `screens::in_control`, and let go of their controls; the pause
//! menu does not open. The player module frees the pointer.

use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use super::atlas::Atlas;
use super::view;
use crate::audio::AudioCue;
use crate::menus::Menu;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Location, Travel};
use crate::universe::Universe;

/// Whether the computer screen is open. The game keeps running under it.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Computer {
    /// The screen is hidden.
    #[default]
    Closed,
    /// The screen is shown.
    Open,
}

/// What decides whether the computer opens or closes.
#[derive(SystemParam)]
struct Seat<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    settings: Res<'w, Persistent<Settings>>,
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

    /// Returns whether the key of the computer and Escape were pressed this frame.
    fn pressed(&self) -> (bool, bool) {
        let key = self.settings.controls.keys.computer;
        (
            self.keys.just_pressed(key),
            self.keys.just_pressed(KeyCode::Escape),
        )
    }
}

/// Returns the state the screen moves to, if any: it opens on its key while seated, and closes
/// on the same key, on Escape or when the player leaves the seat.
pub(super) fn next_state(open: bool, seated: bool, key: bool, escape: bool) -> Option<Computer> {
    match (open, seated) {
        (true, false) => Some(Computer::Closed),
        (true, true) if key || escape => Some(Computer::Closed),
        (false, true) if key => Some(Computer::Open),
        _ => None,
    }
}

/// Opens and closes the computer. Closing on its key sounds a cancel; Escape already does.
///
/// The state changes on the next frame. The pause menu opens only while the computer is
/// closed, so the Escape that closes the screen does not also pause the game.
fn toggle(
    seat: Seat,
    state: Res<State<Computer>>,
    mut next: ResMut<NextState<Computer>>,
    mut cues: MessageWriter<AudioCue>,
) {
    let open = *state.get() == Computer::Open;
    let (key, escape) = seat.pressed();
    let Some(wanted) = next_state(open, seat.taken(), key, escape) else {
        return;
    };
    next.set(wanted);
    if key && !escape {
        cues.write(match wanted {
            Computer::Open => AudioCue::UiConfirm,
            Computer::Closed => AudioCue::UiCancel,
        });
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
        .add_systems(Update, toggle.before(super::Input))
        .add_systems(OnEnter(Computer::Open), view::spawn)
        .add_systems(OnExit(Computer::Open), leave);
}
