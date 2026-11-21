//! The pack and the workshop: one screen over the blurred game, opened on its key on foot.
//!
//! - The pack pane on the left lists what the robot carries, grouped by kind, each row with its
//!   mass and the wear of a tool.
//! - The workshop pane on the right asks for an intent and shows the solver's three proposals as
//!   tool cards: a silhouette, the one reason that matters, what it takes from the pack, which the
//!   pack pane lights, and its numbers as bars. Make takes the focused one.
//! - The hotbar runs under both panes, so a made tool is seen going into its slot.
//! - [`hotbar`] draws the six slots at the bottom of the game while the screen is closed.
//!
//! Every action is a button with its key; the keys and the pointer run the same deeds.

mod bench;
mod cards;
mod click;
mod deeds;
mod glyph;
mod here;
pub mod hotbar;
mod intent;
mod keys;
mod kind;
mod lines;
mod make;
mod offer;
mod open;
mod paint;
mod reason;
mod rows;
mod scene;
mod sheet;
mod slots;
mod tiles;
mod tune;
mod view;

#[cfg(test)]
mod tests_fit;
#[cfg(test)]
mod tests_keys;
#[cfg(test)]
mod tests_lines;
#[cfg(test)]
mod tests_offer;
#[cfg(test)]
mod tests_parts;

pub use bench::{Bench, Pane};

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

/// Registers the open screen: its keys first, then its drawing when what it shows changes, and the
/// clicks on its buttons.
fn screen(app: &mut App) {
    let changed = resource_changed::<Bench>.or_else(resource_changed::<Pack>);
    app.add_systems(OnEnter(Workshop::Open), open::enter)
        .add_observer(click::on_activate)
        .add_systems(
            Update,
            (
                crate::screens::playing.pipe(open::toggle),
                (keys::step, keys::act).run_if(in_state(Workshop::Open)),
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
