//! The words and icons of the four intents, as the workshop shows them: a name, an icon, what the
//! head must be in one short word, and one line that says what the tool does and what it wants.

use crate::craft::build::Intent;
use crate::ui::Icon;

/// Returns the name of `intent`.
pub fn name(intent: Intent) -> &'static str {
    match intent {
        Intent::Cut => "Cut",
        Intent::Pierce => "Pierce",
        Intent::Crush => "Crush",
        Intent::Throw => "Throw",
    }
}

/// Returns the icon of `intent`.
pub fn icon(intent: Intent) -> Icon {
    match intent {
        Intent::Cut => Icon::Cut,
        Intent::Pierce => Icon::Pierce,
        Intent::Crush => Icon::Crush,
        Intent::Throw => Icon::Throw,
    }
}

/// Returns what the head of a tool for `intent` must be, in at most eight letters, so the word
/// fits under the name on the narrowest window.
pub fn need(intent: Intent) -> &'static str {
    match intent {
        Intent::Cut => "an edge",
        Intent::Pierce => "a point",
        Intent::Crush => "a weight",
        Intent::Throw => "a spear",
    }
}

/// Returns one line on what a tool for `intent` does and what it wants, short enough to fit the
/// workshop pane of the narrowest window on one line.
pub fn line(intent: Intent) -> &'static str {
    match intent {
        Intent::Cut => "Slash with a hard edge of flint, obsidian or bone.",
        Intent::Pierce => "Thrust a hard point on a long, stiff shaft.",
        Intent::Crush => "Swing a heavy head of stone or bone. No edge needed.",
        Intent::Throw => "Throw a spear: a light point on a long shaft.",
    }
}
