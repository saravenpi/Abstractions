//! The static layout of the pack screen over the blurred game: the tabs and the count line, the
//! list on the left, the card on the right and the key hints under them. `paint` fills them.

use bevy::prelude::*;

use super::Workshop;
use super::bench::Tab;
use super::layout::LIST;
use crate::fauna::guide::layout::{BETWEEN, CARD, EDGE, FOOT, HEAD};
use crate::ui::{Blurred, Overlay, text, theme};

/// What the footer says on the pack tab.
pub const PACK_HINTS: &str =
    "Up/Down choose  / search  F2 sort  Enter hold  1-6 slot  X drop  Tab workshop  I close";

/// What the footer says on the workshop tab.
pub const MAKE_HINTS: &str =
    "Left/Right intent  Up/Down proposal  H head  J shaft  T treat  Enter make  Tab pack  I close";

/// Marks the name of a tab in the header.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabName(pub Tab);

/// Marks the count line under the tabs.
#[derive(Component)]
pub struct Tally;

/// Marks the column the rows of the list are drawn in.
#[derive(Component)]
pub struct Rows;

/// Marks the column the card is drawn in.
#[derive(Component)]
pub struct Facts;

/// Marks the key hints.
#[derive(Component)]
pub struct Hints;

/// Returns the header: the two tab names and the count line.
fn header() -> impl Bundle {
    let node = Node {
        height: px(HEAD),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let tabs = Node {
        column_gap: px(theme::GAP_L),
        ..default()
    };
    let tab =
        |tab: Tab, name: &'static str| (TabName(tab), text(name, theme::FONT_HEADING, theme::TEXT));
    (
        node,
        children![
            (
                tabs,
                children![tab(Tab::Pack, "Pack"), tab(Tab::Make, "Workshop")]
            ),
            (
                Tally,
                text("", theme::FONT_SMALL, theme::MUTED),
                TextLayout::no_wrap()
            ),
        ],
    )
}

/// Returns the list beside the card.
fn body() -> impl Bundle {
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
    let card = Node {
        width: px(CARD),
        max_width: percent(100),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(),
        ..default()
    };
    (node, children![(Rows, list), (Facts, card)])
}

/// Spawns the empty screen.
pub fn spawn(commands: &mut Commands) {
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
        Hints,
        text(PACK_HINTS, theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    );
    let screen = (Overlay, Blurred, root, BackgroundColor(theme::SCRIM));
    commands.spawn((
        DespawnOnExit(Workshop::Open),
        screen,
        children![header(), body(), (foot, children![hints])],
    ));
}
