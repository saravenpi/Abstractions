//! The hotbar on the open screen: the six slots under both panes, each with its number key, the
//! icon of its tool, the word for it and its wear, the held one lit. A click on a slot puts the
//! selected tool in it, or holds it.

use bevy::prelude::*;

use super::click::Press;
use super::glyph::{ROW, badge};
use super::scene::Slot;
use crate::craft::wield::KEYS;
use crate::fauna::guide::show::clip;
use crate::ui::{Selected, Tone, frame, key_cap, label, line, meter, theme};

/// Characters of small text the word for a tool in a slot shows: "javelin" is the longest.
pub const NAME: usize = 7;

/// Width in pixels of the wear bar of a slot.
pub const WEAR: f32 = 32.0;

/// Returns the node of a slot: a sixth of the row, its key and its words on one line.
fn node() -> Node {
    Node {
        flex_grow: 1.0,
        flex_basis: px(0),
        min_width: px(0),
        height: percent(100),
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        padding: UiRect::horizontal(px(theme::INSET)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Returns the words of `one`: the word for its tool, or "Empty", over its wear.
fn words(one: &Slot) -> impl Bundle {
    let column = Node {
        flex_direction: FlexDirection::Column,
        min_width: px(0),
        ..default()
    };
    let (name, tone) = match &one.name {
        Some(name) => (clip(name, NAME), Tone::Plain),
        None => ("Empty".to_string(), Tone::Muted),
    };
    let drawn = [Visibility::Hidden, Visibility::Inherited][usize::from(one.name.is_some())];
    let wear = (line(), drawn, children![meter(WEAR, 1.0 - one.worn)]);
    (
        column,
        children![label(name, theme::FONT_SMALL, tone), wear],
    )
}

/// Returns the slot `one`, at `at`: its number key, the icon of its tool, then its words, lit when
/// held.
fn slot(at: usize, one: &Slot) -> impl Bundle {
    (
        Press::Slot(at),
        frame(),
        Selected(one.held),
        node(),
        children![
            key_cap(KEYS[at]),
            badge(one.shape, None, ROW, false),
            words(one)
        ],
    )
}

/// Spawns the six slots of `all` under `parent`.
pub fn slots(parent: &mut ChildSpawnerCommands, all: &[Slot]) {
    for (at, one) in all.iter().enumerate() {
        parent.spawn(slot(at, one));
    }
}
