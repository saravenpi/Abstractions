//! The static layout of the guide over the blurred game: the title and the count line, the list
//! of species on the left, the card of the selected one on the right, and the key hints under
//! them. `show` writes the texts.

use bevy::prelude::*;

use super::Guide;
use super::layout::{BETWEEN, CARD, EDGE, FOOT, HEAD, LIST, ROW, measure};
use crate::ui::{
    Blurred, HINT_GAP, Icon, KeyLine, Legend, Overlay, Selected, Tone, bound, frame, icon,
    icon_sized, key_hint, label, scroll_area, text, theme,
};

/// The keys of the guide in its footer: what each does, then the key.
pub const HINTS: [(&str, &str); 4] = [
    ("Choose", "Up Down"),
    ("Page", "PgUp PgDn"),
    ("Scroll", "Wheel"),
    ("Close", "K or Esc"),
];

/// Marks the count line under the title.
#[derive(Component)]
pub struct Tally;

/// A row of the list, counted from the top of the drawn rows.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot(pub usize);

/// Marks the silhouette of a row: the icon of the growth or the body plan of its species.
#[derive(Component)]
pub struct Mark;

/// Marks the icon beside the name on the card: a plant, an animal or a question mark.
#[derive(Component)]
pub struct Sign;

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

/// Returns one row of the list, empty until `show` fills it: the silhouette of the species, its
/// name, and its kind at the right end.
fn row(slot: usize) -> impl Bundle {
    let node = Node {
        width: percent(100),
        height: px(ROW),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        overflow: Overflow::clip(),
        ..default()
    };
    let grow = Node {
        flex_grow: 1.0,
        ..default()
    };
    (
        frame(),
        Selected(false),
        Slot(slot),
        node,
        children![
            (Mark, icon(Icon::Unknown, Tone::Muted)),
            (Called, grow, label("", theme::FONT_BODY, Tone::Plain)),
            (Kind, label("", theme::FONT_SMALL, Tone::Muted))
        ],
    )
}

/// Returns the row of the name of the card: the icon of what the species is, then its name.
fn name() -> impl Bundle {
    let node = Node {
        height: px(theme::ICON),
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        flex_shrink: 0.0,
        ..default()
    };
    let title = (
        Title,
        text("", theme::FONT_BODY, theme::TEXT),
        TextLayout::no_wrap(),
    );
    (
        node,
        children![(Sign, icon(Icon::Guide, Tone::Plain)), title],
    )
}

/// Returns the empty card: its name and what it is, which never scroll, over its facts, which
/// scroll under the pointer when they are taller than the room.
fn card() -> impl Bundle {
    let node = Node {
        width: px(CARD),
        max_width: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let facts = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        margin: UiRect::top(px(theme::GAP_S)),
        ..default()
    };
    let said = (
        Subtitle,
        text("", theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
    );
    (
        node,
        children![name(), said, scroll_area(children![(Facts, facts)])],
    )
}

/// Returns the title of the screen: the icon of the guide and its name.
fn title() -> impl Bundle {
    let node = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        ..default()
    };
    let symbol = icon_sized(Icon::Guide, Tone::Plain, theme::FONT_HEADING);
    let words = text("Field guide", theme::FONT_HEADING, theme::TEXT);
    (node, children![symbol, words])
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
            title(),
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

/// Returns the footer: the key hints of the guide, the close key following its binding.
fn footer() -> impl Bundle {
    let foot = Node {
        min_height: px(FOOT),
        flex_shrink: 0.0,
        align_items: AlignItems::FlexEnd,
        ..default()
    };
    let row = Node {
        align_items: AlignItems::Center,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(HINT_GAP),
        row_gap: px(theme::S1),
        ..default()
    };
    let close = KeyLine::new("{} or Esc", [bound(|keys| keys.guide)]);
    let pairs = children![
        key_hint(HINTS[0].0, HINTS[0].1),
        key_hint(HINTS[1].0, HINTS[1].1),
        key_hint(HINTS[2].0, HINTS[2].1),
        key_hint(HINTS[3].0, close),
    ];
    (foot, children![(Legend, row, pairs)])
}

/// Spawns the guide for a window of `window` logical pixels, with as many list rows as fit.
pub fn spawn(window: Vec2, commands: &mut Commands) {
    let root = Node {
        width: vw(100),
        height: vh(100),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(EDGE)),
        ..default()
    };
    let screen = (Overlay, Blurred, root, BackgroundColor(theme::SCRIM));
    commands.spawn((
        DespawnOnExit(Guide::Open),
        screen,
        children![header(), body(measure(window).rows), footer()],
    ));
}
