//! What a click on the screen does: every button of the screen carries a [`Press`], and the one
//! observer runs it through the same deeds the keys run.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::deeds::Deed;
use crate::craft::build::Intent;

/// What a button of the screen does when it is activated.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// Selects the row of the pack at this index.
    Row(usize),
    /// Asks the workshop for this intent.
    Intent(Intent),
    /// Focuses the tool card at this index.
    Card(usize),
    /// Holds the selected tool.
    Hold,
    /// Drops the selected material of the pack; a tool stays.
    Drop,
    /// Makes the focused tool.
    Make,
    /// Swaps the head material of the focused tool for the next one carried.
    Head,
    /// Swaps the shaft material of the focused tool for the next one carried.
    Shaft,
    /// Treats the head material of the focused tool.
    Treat,
    /// Puts the selected or the new tool in the hotbar slot at this index, or holds the slot.
    Slot(usize),
    /// Closes the screen.
    Close,
}

/// Runs the press of the activated button, if it carries one.
pub(super) fn on_activate(event: On<Activate>, presses: Query<&Press>, mut deed: Deed) {
    if let Ok(press) = presses.get(event.entity) {
        deed.press(*press);
    }
}
