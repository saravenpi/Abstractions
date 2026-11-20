//! What the open screen keeps between frames: the tab, the list window, the filter and the order,
//! and the intent, proposal and swaps of the workshop.

use bevy::prelude::*;

use crate::craft::build::Intent;
use crate::ui::Window;

/// The two tabs of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// What the robot carries.
    #[default]
    Pack,
    /// Making a tool.
    Make,
}

impl Tab {
    /// Returns the other tab.
    pub fn next(self) -> Tab {
        match self {
            Tab::Pack => Tab::Make,
            Tab::Make => Tab::Pack,
        }
    }
}

/// How the rows of the pack are ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Tools first, as they were made, then materials as they were taken.
    #[default]
    Kept,
    /// By name, from A to Z.
    Name,
    /// Heaviest first.
    Mass,
}

impl Sort {
    /// Returns the next order.
    pub fn next(self) -> Sort {
        match self {
            Sort::Kept => Sort::Name,
            Sort::Name => Sort::Mass,
            Sort::Mass => Sort::Kept,
        }
    }

    /// Returns the name of the order as the header shows it: nothing for the kept order.
    pub fn label(self) -> &'static str {
        match self {
            Sort::Kept => "",
            Sort::Name => " by name",
            Sort::Mass => " by mass",
        }
    }
}

/// The state of the open screen.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Bench {
    /// The open tab.
    pub tab: Tab,
    /// The selected and first drawn rows of the pack.
    pub window: Window,
    /// Rows of the list drawn.
    pub rows: usize,
    /// Characters of small text that fit across the card.
    pub chars: usize,
    /// The letters typed to narrow the pack.
    pub filter: String,
    /// Whether letters go into the filter.
    pub typing: bool,
    /// The order of the pack.
    pub sort: Sort,
    /// The intent asked of the workshop.
    pub intent: Intent,
    /// The chosen proposal: lightest, hardest-hitting, most durable.
    pub pick: usize,
    /// How many materials on the shaft and the head were swapped from the proposal, in stock order.
    pub swap: [usize; 2],
    /// What the last action did, in words.
    pub note: String,
}

impl Default for Bench {
    /// Returns the pack tab with nothing chosen, the workshop asked to cut.
    fn default() -> Self {
        Self {
            tab: Tab::Pack,
            window: Window::default(),
            rows: 1,
            chars: 40,
            filter: String::new(),
            typing: false,
            sort: Sort::Kept,
            intent: Intent::Cut,
            pick: 0,
            swap: [0, 0],
            note: String::new(),
        }
    }
}

/// Every intent in the order the workshop steps through them.
pub const INTENTS: [Intent; 4] = [Intent::Cut, Intent::Pierce, Intent::Crush, Intent::Throw];

/// Returns the name of `intent` as the workshop shows it.
pub fn intent_name(intent: Intent) -> &'static str {
    match intent {
        Intent::Cut => "Cut",
        Intent::Pierce => "Pierce",
        Intent::Crush => "Crush",
        Intent::Throw => "Throw",
    }
}

/// Returns the intent `by` places after `intent`, wrapping around.
pub fn step_intent(intent: Intent, by: isize) -> Intent {
    let at = INTENTS.iter().position(|i| *i == intent).unwrap_or(0) as isize;
    INTENTS[(at + by).rem_euclid(INTENTS.len() as isize) as usize]
}
