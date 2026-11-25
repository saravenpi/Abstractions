//! The tool cards of the workshop pane drawn: the three cards of the chosen intent, or the empty
//! state that says what the pack lacks.
//!
//! A card shows the icon of the tool, its name and which of the three it is, the one reason
//! that matters, and once focused what it takes from the pack and its four bars.

use bevy::prelude::*;

use super::click::Press;
use super::glyph::{CARD, badge};
use super::intent;
use super::models::ItemPreview;
use super::render3d::preview_badge;
use super::scene::Tool;
use crate::craft::build::Intent;
use crate::ui::{Icon, Reveal, Tone, card, empty_state, icon_slot, label, line, stat, text, theme};

/// Least width in pixels of the words of a card: four columns of the grid. A card narrower than
/// its picture, its words and its bars puts the bars on a line of their own.
pub const WORDS: f32 = theme::span(4);

/// Returns the node of a block shown only on the focused card.
fn focused_only(focused: bool, node: Node) -> Node {
    Node {
        display: [Display::None, Display::Flex][usize::from(focused)],
        ..node
    }
}

/// Returns a node that takes the room left on its line, and lets its text wrap there.
fn grow() -> Node {
    Node {
        flex_grow: 1.0,
        min_width: px(0),
        ..default()
    }
}

/// Returns the first line of `tool`: its name, then which of the three it is.
fn name_line(tool: &Tool) -> impl Bundle {
    let name = label(tool.name.clone(), theme::FONT_BODY, Tone::Plain);
    let kind = label(tool.kind, theme::FONT_SMALL, Tone::Muted);
    (line(), children![(name, grow()), kind])
}

/// Returns the line of the reason of `tool`: a check for a strength, a warning for a flaw.
fn reason_line(tool: &Tool) -> impl Bundle {
    let sign = [Icon::Check, Icon::Danger][usize::from(tool.reason.bad)];
    let words = text(tool.reason.words.clone(), theme::FONT_SMALL, theme::TEXT);
    (
        line(),
        children![icon_slot(Some(sign), Tone::Plain), (words, grow())],
    )
}

/// Returns the words of `tool`: its name and kind, its reason, and on the focused card what it
/// takes from the pack.
fn words(tool: &Tool) -> impl Bundle {
    let column = Node {
        flex_grow: 1.0,
        flex_basis: px(WORDS),
        min_width: px(WORDS),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let parts = text(tool.parts.clone(), theme::FONT_SMALL, theme::TEXT_2);
    (
        column,
        children![
            name_line(tool),
            reason_line(tool),
            (parts, focused_only(tool.focused, grow()))
        ],
    )
}

/// Returns the four bars of `tool`, shown on the focused card only.
fn bars(tool: &Tool) -> impl Bundle {
    let column = Node {
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        ..default()
    };
    let [a, b, c, d] = tool.gauges.clone();
    (
        focused_only(tool.focused, column),
        children![
            stat(a.label, a.share, a.value),
            stat(b.label, b.share, b.value),
            stat(c.label, c.share, c.value),
            stat(d.label, d.share, d.value),
        ],
    )
}

/// Returns the card of `tool`, the one at `at`.
fn tool_card(at: usize, tool: &Tool) -> impl Bundle {
    let shape = tool.shape;
    let icon = SpawnWith(move |parent: &mut ChildSpawner| {
        if let Some(look) = shape.filter(|l| l.figure.is_some()) {
            parent.spawn(preview_badge(ItemPreview::Tool(look), CARD, false));
        } else {
            parent.spawn(badge(shape, None, CARD, false));
        }
    });
    (
        Press::Card(at),
        card(tool.focused),
        children![
            (Node::default(), Children::spawn(icon)),
            words(tool),
            bars(tool)
        ],
    )
}

/// Spawns the cards of `tools` under `parent`, the focused one kept in view, or the empty state
/// of `chosen` with what the pack lacks.
pub fn cards(parent: &mut ChildSpawnerCommands, tools: &Result<Vec<Tool>, String>, chosen: Intent) {
    let Ok(tools) = tools else {
        let why = tools.as_ref().err().cloned().unwrap_or_default();
        let title = format!(
            "Nothing to {} with yet",
            intent::name(chosen).to_lowercase()
        );
        parent.spawn(empty_state(intent::icon(chosen), &title, &why));
        return;
    };
    let column = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::S2),
        ..default()
    };
    parent.spawn(column).with_children(|inner| {
        for (at, tool) in tools.iter().enumerate() {
            let mut made = inner.spawn(tool_card(at, tool));
            if tool.focused {
                made.insert(Reveal);
            }
        }
    });
}
