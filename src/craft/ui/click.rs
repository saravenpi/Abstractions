//! What a click on the screen does: every button of the screen carries a [`Press`], and the one
//! observer runs it through the same deeds the keys run.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::bench::Pane;
use super::deeds::Deed;
use crate::craft::armor::ArmorSlot;
use crate::craft::build::Intent;
use crate::craft::build::catalog::Category;

/// Marker on an equipped armor slot button that unequips `0` back into the pack grid when clicked.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmorPress(pub ArmorSlot);

/// What a button of the screen does when it is activated.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// Opens the tab of this pane.
    Pane(Pane),
    /// Clicks an inventory grid or hotbar slot (`0..30`): select, swap/merge, or Shift quick-move.
    GridSlot(usize),
    /// Sorts and compacts stacks inside the 24-slot pack grid.
    SortPack,
    /// Selects a crafting category tab in the Workshop.
    Category(Category),
    /// Focuses an explicit crafting recipe in the current category list.
    Recipe(usize),
    /// Crafts `N` batches of the focused recipe (`0` means Craft Max).
    CraftBatch(u16),
    /// Cycles the mineral used when crafting bullets.
    CycleMineral,
    /// Asks the workshop for this intent.
    Intent(Intent),
    /// Focuses the tool card at this index.
    Card(usize),
    /// Holds the selected tool.
    Hold,
    /// Puts the selected tool or stack of the pack down on the ground, half the stack with Shift.
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
pub(super) fn on_activate(
    event: On<Activate>,
    presses: Query<&Press>,
    armors: Query<&ArmorPress>,
    mut deed: Deed,
) {
    if let Ok(press) = presses.get(event.entity) {
        deed.press(*press);
    } else if let Ok(&ArmorPress(slot)) = armors.get(event.entity) {
        deed.click_armor_slot(slot);
    }
}

/// Splits a stack in half into a free slot when right-clicking a hovered grid slot.
pub(super) fn on_right_click(
    buttons: Res<ButtonInput<MouseButton>>,
    presses: Query<(&Press, &Hovered)>,
    mut deed: Deed,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    for (&press, hovered) in &presses {
        if hovered.get()
            && let Press::GridSlot(at) = press
        {
            deed.split_grid_slot(at);
            return;
        }
    }
}
