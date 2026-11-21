//! Drawing the stat rows of the card: an icon, a label, a value with its unit and a few words,
//! the plain meaning of a group, and the More row that shows the rarer facts.

use bevy::prelude::*;

use super::course::Act;
use crate::ui::{Icon, Tone, caps, entry, frame, icon, text, theme};

/// Width in pixels of the column of comparisons and meanings at the right end of a stat.
const NOTE_WIDTH: f32 = 112.0;

/// Returns one line of small text of the given colour that never wraps.
fn small_text(words: &str, color: Color) -> impl Bundle {
    (
        text(words.to_string(), theme::FONT_SMALL, color),
        TextLayout::no_wrap(),
    )
}

/// Spawns one stat of the card under `parent`: its icon, its label at the left, its value, and
/// its note in a column of its own at the right end, when it has one.
pub(super) fn stat(
    parent: &mut ChildSpawnerCommands,
    (symbol, label): (Icon, &str),
    words: (&str, &str),
) {
    let line = Node {
        width: percent(100),
        height: px(theme::LINE_SMALL),
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        overflow: Overflow::clip(),
        ..default()
    };
    let (value, note) = words;
    let grow = Node {
        flex_grow: 1.0,
        min_width: px(0),
        overflow: Overflow::clip(),
        ..default()
    };
    let small = small_text;
    let tail = Node {
        width: px(NOTE_WIDTH),
        justify_content: JustifyContent::FlexEnd,
        ..default()
    };
    let mut row = parent.spawn(line);
    row.with_children(|row| {
        row.spawn(icon(symbol, Tone::Muted));
        row.spawn((grow, children![small(label, theme::TEXT_2)]));
        row.spawn(small(value, theme::TEXT));
        if !note.is_empty() {
            row.spawn((tail, children![small(note, theme::TEXT_2)]));
        }
    });
}

/// Spawns the heading of a group under `parent`: its icon and its name in spaced capitals, with
/// `gap` pixels over it to set the group apart from the one above.
pub(super) fn group(parent: &mut ChildSpawnerCommands, (symbol, name): (Icon, &str), gap: f32) {
    let line = Node {
        height: px(theme::LINE_SMALL),
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        margin: UiRect::top(px(gap)),
        ..default()
    };
    parent.spawn((line, children![icon(symbol, Tone::Muted), caps(name)]));
}

/// Spawns the plain meaning of a group under `parent`, in line with the labels of its stats.
pub(super) fn meaning(parent: &mut ChildSpawnerCommands, words: &str) {
    let node = Node {
        margin: UiRect::left(px(theme::ICON + theme::S2)),
        ..default()
    };
    parent.spawn((text(words, theme::FONT_SMALL, theme::TEXT_2), node));
}

/// Spawns the row that shows or hides the rarer facts, which reads More while they are hidden
/// and Less while they show.
pub(super) fn more(parent: &mut ChildSpawnerCommands, open: bool) {
    let (symbol, words) = match open {
        true => (Icon::ExpandLess, "Less"),
        false => (Icon::ExpandMore, "More"),
    };
    let node = Node {
        height: px(theme::ROW_COMPACT),
        align_items: AlignItems::Center,
        ..default()
    };
    let shown = entry(symbol, words, theme::FONT_SMALL, Tone::Muted);
    parent.spawn((frame(), Act::More, node, children![shown]));
}
