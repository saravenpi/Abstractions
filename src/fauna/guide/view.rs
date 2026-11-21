//! The static layout of the guide over the blurred game: the title and the count line, the list
//! of species on the left, the card of the selected one on the right, and the key hints under
//! them. `show` writes the texts.

use bevy::prelude::*;

use super::Guide;
use super::layout::{BETWEEN, CARD, EDGE, FOOT, HEAD, LIST, ROW, measure};
use crate::ui::{
    Blurred, KeyLine, Overlay, Selected, Tone, bound, frame, key_text, label, text, theme,
};

/// What the footer says, with a `{}` for the key of the field guide.
const HINTS: &str = "Up and Down to choose  -  {} or Esc to close";

/// Marks the count line under the title.
#[derive(Component)]
pub struct Tally;

/// A row of the list, counted from the top of the drawn rows.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot(pub usize);

/// Marks the name text of a row.
#[derive(Component)]
pub struct Called;

/// Marks the kind text of a row.
#[derive(Component)]
pub struct Kind;

/// Marks the name on the card.
#[derive(Component)]
pub struct Title;

/// Marks the line under the name on the card.
#[derive(Component)]
pub struct Subtitle;

/// Marks the column the facts of the card are written in.
#[derive(Component)]
pub struct Facts;

/// Returns one row of the list, empty until `show` fills it.
fn row(slot: usize) -> impl Bundle {
    let node = Node {
        width: percent(100),
        height: px(ROW),
        flex_shrink: 0.0,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        ..default()
    };
    (
        frame(),
        Selected(false),
        Slot(slot),
        node,
        children![
            (Called, label("", theme::FONT_BODY, Tone::Plain)),
            (Kind, label("", theme::FONT_SMALL, Tone::Muted))
        ],
    )
}

/// Returns the empty card.
fn card() -> impl Bundle {
    let node = Node {
        width: px(CARD),
        max_width: percent(100),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(),
        ..default()
    };
    let facts = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        margin: UiRect::top(px(theme::GAP_S)),
        ..default()
    };
    (
        node,
        children![
            (
                Title,
                text("", theme::FONT_BODY, theme::TEXT),
                TextLayout::no_wrap()
            ),
            (
                Subtitle,
                text("", theme::FONT_SMALL, theme::TEXT_2),
                TextLayout::no_wrap()
            ),
            (Facts, facts),
        ],
    )
}

/// Returns the header: the title and the count line.
fn header() -> impl Bundle {
    let node = Node {
        height: px(HEAD),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        ..default()
    };
    (
        node,
        children![
            text("Field guide", theme::FONT_HEADING, theme::TEXT),
            (
                Tally,
                text("", theme::FONT_SMALL, theme::TEXT_2),
                TextLayout::no_wrap()
            ),
        ],
    )
}

/// Returns the list beside the card, with `rows` rows.
fn body(rows: usize) -> impl Bundle {
    let node = Node {
        flex_grow: 1.0,
        min_height: px(0),
        column_gap: px(BETWEEN),
        overflow: Overflow::clip(),
        ..default()
    };
    let list = Node {
        width: px(LIST),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let slots = Children::spawn(SpawnIter((0..rows).map(row)));
    (node, children![(list, slots), card()])
}

/// Spawns the guide for a window of `window` logical pixels, with as many list rows as fit.
pub fn spawn(window: Vec2, commands: &mut Commands) {
    let root = Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(EDGE)),
        ..default()
    };
    let foot = Node {
        height: px(FOOT),
        flex_shrink: 0.0,
        align_items: AlignItems::FlexEnd,
        ..default()
    };
    let hints = (
        foot,
        children![key_text(
            KeyLine::new(HINTS, [bound(|keys| keys.guide)]),
            theme::FONT_SMALL,
            theme::TEXT_2
        )],
    );
    let screen = (Overlay, Blurred, root, BackgroundColor(theme::SCRIM));
    commands.spawn((
        DespawnOnExit(Guide::Open),
        screen,
        children![header(), body(measure(window).rows), hints],
    ));
}
