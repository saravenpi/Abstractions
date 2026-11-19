//! Opening and closing the map screen: its key opens it everywhere, and the key of the board
//! computer opens it while seated in the ship.
//!
//! While the screen is open the keyboard and the mouse belong to it. The ship and the player
//! look at [`Computer`], or at `screens::in_control`, and let go of their controls, but the
//! throttle and the autopilot keep the ship flying; the pause menu does not open. The player
//! module frees the pointer.

use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use super::atlas::Atlas;
use super::board::Board;
use super::hold::Hold;
use super::typing::Typing;
use super::view;
use crate::audio::AudioCue;
use crate::menus::Menu;
use crate::screens::{Screen, WarmUp};
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Location, Travel};
use crate::universe::Universe;

/// Whether the map screen is open. The game keeps running under it.
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
    warm: Option<Res<'w, State<WarmUp>>>,
    location: Option<Res<'w, Location>>,
}

/// Whether the player is in the world with no menu open, and seated in the ship.
pub(super) type Where = (bool, bool);

/// Which keys were pressed this frame: the map key, the key of the board computer, Escape.
pub(super) type Pressed = (bool, bool, bool);

impl Seat<'_> {
    /// Returns whether the player is in the world past the boot screen with no menu open, and
    /// whether they are seated in the ship.
    fn place(&self) -> Where {
        let hidden = self
            .warm
            .as_deref()
            .is_some_and(|warm| *warm.get() == WarmUp::Hidden);
        let menu = *self.menu.get() == Menu::None;
        let playing = *self.screen.get() == Screen::Gameplay && menu && !hidden;
        let seated = self.location.as_deref().map(|location| location.travel);
        (playing, playing && seated == Some(Travel::InShip))
    }

    /// Returns whether the map key, the key of the board computer and Escape were pressed.
    fn pressed(&self) -> Pressed {
        let keys = self.settings.controls.keys;
        (
            self.keys.just_pressed(keys.map),
            self.keys.just_pressed(keys.computer),
            self.keys.just_pressed(KeyCode::Escape),
        )
    }
}

/// Returns the state the screen moves to, if any: it opens on the map key anywhere in the
/// world and on the key of the board computer while seated, and closes on either key, on
/// Escape, or when the world or a menu takes the screen.
pub(super) fn next_state(
    open: bool,
    (playing, seated): Where,
    (map, tab, escape): Pressed,
) -> Option<Computer> {
    match open {
        true if !playing || map || tab || escape => Some(Computer::Closed),
        false if playing && (map || (tab && seated)) => Some(Computer::Open),
        _ => None,
    }
}

/// Opens and closes the map. Closing on a key sounds a cancel; Escape already does. While a
/// search is typed or a favourite named, Escape and the map key type, and the screen stays.
/// While a menu is open or something is selected, Escape lets go of it first.
///
/// The state changes on the next frame. The pause menu opens only while the map is closed, so
/// the Escape that closes the screen does not also pause the game.
fn toggle(
    seat: Seat,
    state: Res<State<Computer>>,
    mut next: ResMut<NextState<Computer>>,
    (board, typing, mut hold): (Res<Board>, Res<Typing>, Hold),
    mut cues: MessageWriter<AudioCue>,
) {
    let open = *state.get() == Computer::Open;
    let (map, tab, escape) = seat.pressed();
    let free = typing.is_off() && board.filter.is_empty();
    let held = open && escape && free && hold.held();
    if held {
        hold.release();
    }
    let pressed = (
        map && (typing.is_off() || !open),
        tab,
        escape && free && !held,
    );
    let Some(wanted) = next_state(open, seat.place(), pressed) else {
        return;
    };
    next.set(wanted);
    if (pressed.0 || pressed.1) && !pressed.2 {
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
