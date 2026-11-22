//! Key hints and texts that name keys: the one way the interface shows a key beside what it does,
//! following the keyboard layout of the player and the key bindings. The label comes first in its
//! own tone, then the name of the key, without brackets, in the secondary text colour and
//! [`KEY_GAP`] away. A legend is a row of hints [`HINT_GAP`] apart, so each pair reads as one.

use bevy::prelude::*;

use super::item::label;
use super::key_line::{KeyLine, key_label};
use super::style::Tone;
use super::theme;
use super::widgets::text;
use crate::settings::{Keymap, Labels, Persistent, Settings};

/// Pixels between a label and the name of its key.
pub const KEY_GAP: f32 = theme::S3;

/// Pixels between two hints of a legend.
pub const HINT_GAP: f32 = theme::S5;

/// Marks the text that names a key.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct KeyName;

/// Marks the text that says what the key of a hint does.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HintLabel;

/// Marks a row of key hints.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Legend;

/// The name of a key in the secondary text colour, whatever the look of the button around it.
/// It has no box and no brackets: its colour and the gap before it set it apart. Every action
/// shows its key: pair this with the label of the action, [`KEY_GAP`] after it.
pub fn key_cap(key: impl Into<KeyLine>) -> impl Bundle {
    (
        KeyName,
        key_label(key, theme::FONT_SMALL, Tone::Key),
        TextColor(theme::TEXT_2),
    )
}

/// One hint that is not a button: what a key does, then the name of the key.
pub fn key_hint(words: impl Into<String>, key: impl Into<KeyLine>) -> impl Bundle {
    let node = Node {
        align_items: AlignItems::Center,
        column_gap: px(KEY_GAP),
        flex_shrink: 0.0,
        ..default()
    };
    let said = (
        HintLabel,
        text(words, theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
    );
    (node, children![said, key_cap(key)])
}

/// Returns `pairs` of a label and a key name as the owned hints a legend takes.
pub fn hints(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    let owned = |(words, key): &(&str, &str)| (words.to_string(), key.to_string());
    pairs.iter().map(owned).collect()
}

/// The hints of `hints` in a row on `node`, which wraps to more lines when it is too narrow.
fn row(node: Node, hints: Vec<(String, String)>) -> impl Bundle {
    let node = Node {
        align_items: AlignItems::Center,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(HINT_GAP),
        row_gap: px(theme::S1),
        ..node
    };
    let pairs = hints.into_iter().map(|(words, key)| {
        let node = Node {
            align_items: AlignItems::Center,
            column_gap: px(KEY_GAP),
            flex_shrink: 0.0,
            ..default()
        };
        let said = (
            HintLabel,
            text(words, theme::FONT_SMALL, theme::TEXT),
            TextLayout::no_wrap(),
        );
        let cap = (
            KeyName,
            label(key, theme::FONT_SMALL, Tone::Key),
            TextColor(theme::TEXT_2),
        );
        (node, children![said, cap])
    });
    (Legend, node, Children::spawn(SpawnIter(pairs)))
}

/// A legend: each label of `hints` with its key, in a row that wraps.
pub fn legend(hints: Vec<(String, String)>) -> impl Bundle {
    row(Node::default(), hints)
}

/// A legend pinned to the bottom of the screen, in line with the content of a screen: the same
/// side margin, and one edge margin from the bottom.
pub fn footer(hints: Vec<(String, String)>) -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        left: percent(5),
        right: percent(5),
        bottom: px(theme::EDGE),
        ..default()
    };
    row(node, hints)
}

/// Run condition: a label, a binding or a line changed.
pub(super) fn stale(
    labels: Labels,
    settings: Option<Res<Persistent<Settings>>>,
    lines: Query<(), Changed<KeyLine>>,
) -> bool {
    let rebound = settings.is_some_and(|settings| settings.is_changed());
    labels.is_changed() || rebound || !lines.is_empty()
}

/// Writes every text that names keys.
pub(super) fn write(
    labels: Labels,
    settings: Option<Res<Persistent<Settings>>>,
    mut lines: Query<(&KeyLine, &mut Text)>,
) {
    let map = settings.map_or_else(Keymap::default, |settings| settings.controls.keys);
    let labels = labels.get();
    for (line, mut text) in &mut lines {
        text.set_if_neq(Text(line.words(&map, &labels)));
    }
}
