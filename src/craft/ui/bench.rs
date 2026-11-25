//! What the open screen keeps between frames: the pane the keys drive, the cursor of the pack, and
//! the intent, proposal and swaps of the workshop.

use bevy::prelude::*;

use crate::craft::build::Intent;
use crate::craft::build::catalog::Category;
use crate::craft::matter::Mineral;

/// The two panes of the screen, side by side. The keys drive one of them at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane {
    /// What the robot carries, on the left.
    #[default]
    Pack,
    /// Making a tool, on the right.
    Workshop,
}

impl Pane {
    /// Returns the other pane.
    pub fn next(self) -> Pane {
        match self {
            Pane::Pack => Pane::Workshop,
            Pane::Workshop => Pane::Pack,
        }
    }
}

/// The state of the open screen.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Bench {
    /// The pane the keys drive.
    pub pane: Pane,
    /// The selected row of the pack, counted over every group from the top.
    pub cursor: usize,
    /// The intent asked of the workshop.
    pub intent: Intent,
    /// The focused proposal: lightest, hardest-hitting, most durable.
    pub pick: usize,
    /// How many materials on the shaft and the head were swapped from the proposal, in stock order.
    pub swap: [usize; 2],
    /// The id of the tool the workshop made last, which the number keys put in a slot.
    pub made: Option<u32>,
    /// What the last action did, in words.
    pub note: String,
    /// Explicitly clicked 24+6 grid slot (`0..30`), overriding row-list cursor when set.
    pub selected_slot: Option<usize>,
    /// Grid slot currently picked up for pointer swap/merge (`0..30`).
    pub picked_slot: Option<usize>,
    /// Active workshop filter category tab.
    pub category: Category,
    /// Selected recipe index within the active workshop category list.
    pub recipe_idx: usize,
    /// Mineral chosen for crafting mineral bullets.
    pub bullet_mineral: Mineral,
}

impl Default for Bench {
    /// Returns the pack pane with its first row selected, the workshop ready for crafting.
    fn default() -> Self {
        Self {
            pane: Pane::Pack,
            cursor: 0,
            intent: Intent::Cut,
            pick: 0,
            swap: [0, 0],
            made: None,
            note: String::new(),
            selected_slot: None,
            picked_slot: None,
            category: Category::All,
            recipe_idx: 0,
            bullet_mineral: Mineral::Ironstone,
        }
    }
}

/// Every intent in the order the workshop shows them.
pub const INTENTS: [Intent; 4] = [Intent::Cut, Intent::Pierce, Intent::Crush, Intent::Throw];

/// Returns the intent `by` places after `intent`, wrapping around.
pub fn step_intent(intent: Intent, by: isize) -> Intent {
    let at = INTENTS.iter().position(|i| *i == intent).unwrap_or(0) as isize;
    INTENTS[(at + by).rem_euclid(INTENTS.len() as isize) as usize]
}
