//! The field guide: a full-screen list of the species of the planet underfoot over the blurred
//! game, with a card of plain facts for the selected one.
//!
//! - [`seen`] keeps which species the player has seen, and marks them from what is near.
//! - [`book`] lists the species of the planet, plants first, unseen ones as unknown.
//! - [`card`] and [`relations`] put the facts of one species in words, in groups.
//! - [`layout`] measures the guide in a window; [`view`] spawns it and `show` writes it.
//! - [`open`] opens it on its key, closes it, and moves the selection.

pub mod book;
pub mod card;
pub mod layout;
pub mod open;
pub mod relations;
#[cfg(test)]
mod rig;
pub mod seen;
mod show;
pub mod view;
pub mod words;

#[cfg(test)]
mod tests_book;
#[cfg(test)]
mod tests_fit;
#[cfg(test)]
mod tests_seen;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub use seen::Seen;

use crate::space::on_foot;

/// Seconds between two looks for creatures near the robot.
const LOOK: f32 = 0.25;

/// Whether the field guide is open. The game keeps running under it.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Guide {
    /// The guide is hidden.
    #[default]
    Closed,
    /// The guide is shown.
    Open,
}

/// Run condition: the field guide is open.
pub fn shown(state: Option<Res<State<Guide>>>) -> bool {
    state.is_some_and(|state| *state.get() == Guide::Open)
}

/// Registers the systems of the open guide: its keys, the wheel and its texts.
fn screen(app: &mut App) {
    let texts = (show::list, show::card_of).run_if(resource_changed::<open::Shelf>);
    let shown = (
        open::refresh.run_if(resource_changed::<Seen>),
        open::step,
        open::wheel,
        texts,
    );
    app.add_systems(OnEnter(Guide::Open), open::enter)
        .add_observer(open::pick)
        .add_systems(Update, shown.chain().run_if(in_state(Guide::Open)));
}

/// Registers the field guide and the marking of seen species.
pub(crate) fn plugin(app: &mut App) {
    let look = on_timer(Duration::from_secs_f32(LOOK));
    app.init_state::<Guide>()
        .init_resource::<Seen>()
        .init_resource::<open::Shelf>()
        .add_systems(OnExit(crate::screens::Screen::Gameplay), seen::forget)
        .add_systems(
            Update,
            (
                seen::sight.run_if(on_foot.and_then(look)),
                crate::screens::playing
                    .pipe(open::toggle)
                    .run_if(in_state(crate::screens::Screen::Gameplay).or_else(shown)),
            ),
        );
    screen(app);
}
