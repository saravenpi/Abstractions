//! Full-page game states: title, loading, the wake-up boot screen and gameplay.

mod boot;
mod gameplay;
#[cfg(test)]
mod hide_tests;
mod loading;
mod slides;
#[cfg(test)]
mod slides_app_tests;
#[cfg(test)]
mod slides_tests;
mod stall;
#[cfg(test)]
mod stall_tests;
#[cfg(test)]
mod tests;
mod title;
#[cfg(test)]
mod title_tests;
mod wake;

use bevy::prelude::*;

pub use gameplay::{HIDE_KEY, HideHud};

use crate::computer::Computer;
use crate::fauna::guide::Guide;
use crate::menus::Menu;

/// The logical screen the game is showing.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    /// Title screen with the main menu on top.
    #[default]
    Title,
    /// World generation around the spawn point.
    Loading,
    /// The player is in the world.
    Gameplay,
}

/// Whether the world is hidden behind the boot screen.
///
/// While `Hidden` the screen is `Screen::Gameplay`: the world is loaded, streamed and drawn,
/// but the boot screen covers it and the player cannot act. It lasts a few seconds after every
/// load. This is the time to draw each material once so that its pipeline exists before the
/// world fades in: use `in_state(WarmUp::Hidden)`, `OnEnter(WarmUp::Hidden)` and
/// `OnExit(WarmUp::Hidden)`.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WarmUp {
    /// No boot screen: the world is visible, or there is no world.
    #[default]
    Idle,
    /// The boot screen covers the loaded world.
    Hidden,
}

/// Run condition: true while the world is drawn: in the world with no menu open, the boot
/// screen included. What only shows the world uses it, the camera and the pose of the body, so
/// the world the boot screen fades away from is already seen from the player, never from where
/// the camera stood before. What the player's hands or the physics drive waits for [`playing`].
pub fn shown(screen: Res<State<Screen>>, menu: Res<State<Menu>>) -> bool {
    *screen.get() == Screen::Gameplay && *menu.get() == Menu::None
}

/// Run condition: true while the player is in the world, past the boot screen, with no menu open.
///
/// The board computer does not stop play: use [`in_control`] for what reads the player's hands.
///
/// An app with no screen state, such as a test of one area, is never playing.
pub fn playing(
    screen: Option<Res<State<Screen>>>,
    menu: Option<Res<State<Menu>>>,
    warm: Option<Res<State<WarmUp>>>,
) -> bool {
    let hidden = warm.is_some_and(|warm| *warm.get() == WarmUp::Hidden);
    let world = screen.is_some_and(|screen| *screen.get() == Screen::Gameplay);
    world && menu.is_none_or(|menu| *menu.get() == Menu::None) && !hidden
}

/// The screens that cover the game with their own keys: the field guide, the pack, and chests.
type Covers<'w> = (
    Option<Res<'w, State<Guide>>>,
    Option<Res<'w, State<crate::craft::ui::Workshop>>>,
    Option<Res<'w, State<crate::craft::chest::ChestScreen>>>,
);

/// Run condition: true while the keyboard and the mouse drive the player or the ship: playing,
/// and the board computer and the field guide are closed.
///
/// While the computer is open the game goes on, so [`playing`] stays true, but keys and mouse
/// belong to the computer screen. Movement, steering, boarding and the cursor grab use this.
pub fn in_control(
    screen: Option<Res<State<Screen>>>,
    menu: Option<Res<State<Menu>>>,
    warm: Option<Res<State<WarmUp>>>,
    computer: Option<Res<State<Computer>>>,
    (guide, bench, chest): Covers,
) -> bool {
    let open = computer.is_some_and(|computer| *computer.get() == Computer::Open);
    let over = crate::fauna::guide::shown(guide)
        || crate::craft::ui::shown(bench)
        || crate::craft::chest::shown(chest);
    playing(screen, menu, warm) && !open && !over
}

/// Registers the screen states and every screen plugin.
pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>().init_state::<WarmUp>();
    app.add_plugins((
        title::plugin,
        loading::plugin,
        stall::plugin,
        boot::plugin,
        gameplay::plugin,
    ));
}
