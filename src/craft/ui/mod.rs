//! The pack and the workshop: one screen over the blurred game, opened on its key on foot.
//!
//! - The pack tab lists what the robot carries, with a card of the selected entry.
//! - The workshop tab asks for an intent and shows the solver's three proposals as differences
//!   against the held tool, with the one reason that matters, and makes the chosen one.
//! - [`hotbar`] draws the six slots at the bottom of the game.

mod bench;
mod card;
mod glyph;
mod here;
pub mod hotbar;
mod keys;
mod layout;
mod lines;
mod offer;
mod open;
mod paint;
mod sheet;
mod view;

#[cfg(test)]
mod tests_lines;
#[cfg(test)]
mod tests_offer;

pub use bench::{Bench, Sort, Tab};

use bevy::prelude::*;

use crate::craft::pack::Pack;

/// Whether the pack screen is open. The game keeps running under it.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Workshop {
    /// The screen is hidden.
    #[default]
    Closed,
    /// The screen is shown.
    Open,
}

/// Run condition: the pack screen is open.
pub fn shown(state: Option<Res<State<Workshop>>>) -> bool {
    state.is_some_and(|state| *state.get() == Workshop::Open)
}

/// Registers the open screen: its keys first, then its texts when what they show changes.
fn screen(app: &mut App) {
    let changed = resource_changed::<Bench>.or_else(resource_changed::<Pack>);
    let keys = (keys::type_filter, keys::step, keys::act, open::wheel);
    app.add_systems(OnEnter(Workshop::Open), open::enter)
        .add_systems(
            Update,
            (
                crate::screens::playing.pipe(open::toggle),
                keys.run_if(in_state(Workshop::Open)),
                paint::draw.run_if(in_state(Workshop::Open).and_then(changed)),
            )
                .chain()
                .run_if(in_state(crate::screens::Screen::Gameplay).or_else(shown)),
        );
}

/// Registers the pack screen and the hotbar.
pub(crate) fn plugin(app: &mut App) {
    app.init_state::<Workshop>().init_resource::<Bench>();
    screen(app);
    hotbar::plugin(app);
}
