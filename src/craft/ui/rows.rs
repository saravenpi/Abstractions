//! Pack row formatting helpers: mass, hotbar slot, grade, and uses text.

use super::scene::Row;
use crate::ui::units::fixed;

/// Characters of body text a name shows: the width of the pack pane less the icon slot.
pub const NAME: usize = 28;

/// Characters of small text the line under a name shows, beside the wear bar.
pub const FACT: usize = 34;

/// Width in pixels of the wear bar of a tool.
pub const WEAR: f32 = 32.0;

/// Returns the line under the name of `row`: its mass, then what the focused card takes from it,
/// or its hotbar slot when equipped, grade and signature.
pub fn fact(row: &Row) -> String {
    let mass = fixed(row.line.mass, 2);
    match (row.uses, row.line.slot) {
        (Some(kg), _) => format!("uses {} of {mass} kg", fixed(kg, 2)),
        (None, Some(slot)) => format!("#{slot}, {mass} kg, {}", row.line.signature),
        (None, None) => format!("{mass} kg, {}", row.line.signature),
    }
}
