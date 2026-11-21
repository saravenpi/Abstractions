//! The tabs of the heading zone and the intent tiles of the workshop drawn. The two tabs, Pack and
//! Workshop, are words of the page title size, the open one lit, then the Tab key that switches
//! them. The four tiles sit side by side, each with the icon of its intent, its name and what its
//! head must be, the chosen one lit.

use bevy::prelude::*;

use super::bench::{INTENTS, Pane};
use super::click::Press;
use super::intent;
use crate::craft::build::Intent;
use crate::ui::{Selected, Tone, frame, icon_slot, key_cap, label, theme};

/// Returns the node of a tile: a share of the row, the icon and the words on the icon line.
fn node() -> Node {
    Node {
        flex_grow: 1.0,
        flex_basis: px(0),
        min_width: px(0),
        height: percent(100),
        align_items: AlignItems::Center,
        column_gap: px(theme::ICON_GAP),
        padding: UiRect::horizontal(px(theme::INSET)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Returns the tile of `intent`, chosen or not: its icon, its name and what its head must be.
fn tile(intent: Intent, chosen: bool) -> impl Bundle {
    let words = Node {
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let name = label(intent::name(intent), theme::FONT_BODY, Tone::Plain);
    let need = label(intent::need(intent), theme::FONT_SMALL, Tone::Muted);
    (
        Press::Intent(intent),
        frame(),
        Selected(chosen),
        node(),
        children![
            icon_slot(Some(intent::icon(intent)), Tone::Plain),
            (words, children![name, need])
        ],
    )
}

/// Spawns the four intent tiles under `parent`, `chosen` lit.
pub fn intents(parent: &mut ChildSpawnerCommands, chosen: Intent) {
    for one in INTENTS {
        parent.spawn(tile(one, one == chosen));
    }
}

/// Returns the tab of `pane` named `name`, lit when `open`.
fn tab(pane: Pane, name: &str, open: bool) -> impl Bundle {
    let node = Node {
        height: px(theme::LINE_HEADING),
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(px(theme::INSET)),
        ..default()
    };
    (
        Press::Pane(pane),
        frame(),
        Selected(open),
        node,
        children![label(name, theme::FONT_HEADING, Tone::Muted)],
    )
}

/// Spawns the two tabs under `parent`, `open` lit, and the key that switches them.
pub fn tabs(parent: &mut ChildSpawnerCommands, open: Pane) {
    parent.spawn(tab(Pane::Pack, "Pack", open == Pane::Pack));
    parent.spawn(tab(Pane::Workshop, "Workshop", open == Pane::Workshop));
    parent.spawn(key_cap("Tab"));
}
