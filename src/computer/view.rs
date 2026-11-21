//! Static layout of the map screen: two translucent panels inside the margins of the screen,
//! over the blurred game. The panel of the map is on the left, with the path of the levels over
//! its top left corner and the progress of the route over its bottom. The side panel on the
//! right holds the search box with its results, the head of the card of the selected result
//! with its Go button, then what scrolls, the facts, the pins and the players, and at its foot
//! the keys of the map.
//!
//! The game keeps running under the screen. Every button of the screen is for the pointer
//! only: hovering one lights it and never moves the keyboard focus, which stays on the
//! selected result.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::layout::{BETWEEN, EDGE, LINE_HEIGHT, MARGIN, PAD, SECTION_GAP, SIDE};
use super::map::field;
use super::open::Computer;
use crate::ui::{Blurred, Overlay, text, theme};

/// What the search box says while nothing is typed.
pub(super) const SEARCH_HINT: &str = "Search";

/// Marks the whole screen.
#[derive(Component)]
pub(super) struct Panel;

/// Marks the room of the map beside the panel, which cuts the chart at its edges.
#[derive(Component)]
pub struct Area;

/// A result button of the pool, counted from the top of the list.
#[derive(Component, Debug, Clone, Copy)]
pub struct Slot(pub usize);

/// Which text of a result a cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The icon that says what the row is, or marks the destination or the place of the ship.
    Mark,
    /// The name.
    Title,
    /// The distance from the ship.
    Distance,
}

/// A text of the result in this slot.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Cell(pub usize, pub Part);

/// Marks the line that names the results and counts them.
#[derive(Component)]
pub(super) struct Summary;

/// Marks the line that shows the course, its distance and the route state.
#[derive(Component)]
pub(super) struct Status;

/// Marks the line that says how far along the route is while it is flown.
#[derive(Component)]
pub(super) struct Action;

/// Marks the search box, which a click makes the place typing goes to.
#[derive(Component)]
pub struct SearchBox;

/// Marks the text of the search box.
#[derive(Component)]
pub(super) struct SearchText;

/// The panel on the right, in the order of what matters: the search with its results, the head
/// of the card with the name of the selected place and its Go button, then what scrolls under
/// them, the facts, the pins and the players, and at the foot the keys. The search, the head
/// and the keys always show whole.
fn side() -> impl Bundle {
    let node = Node {
        width: px(SIDE),
        height: percent(100),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        row_gap: px(SECTION_GAP),
        padding: UiRect::all(px(PAD)),
        overflow: Overflow::clip(),
        ..default()
    };
    (
        node,
        BackgroundColor(theme::PANEL_BLUR),
        children![
            super::results::search(),
            super::readout::card(),
            super::section::body(),
            super::section::legend()
        ],
    )
}

/// A column of lines over the map that shrinks and is cut before it runs under what is at the
/// right end of its row.
fn stack() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::GAP_XS),
        flex_shrink: 1.0,
        min_width: px(0),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// A row over the map with `left` on its left and `right` at its right end, aligned along
/// `align`.
fn ends(align: AlignItems, left: impl Bundle, right: impl Bundle) -> impl Bundle {
    let row = Node {
        width: percent(100),
        justify_content: JustifyContent::SpaceBetween,
        align_items: align,
        column_gap: px(theme::GAP_M),
        ..default()
    };
    (row, children![left, right])
}

/// A line of small text over the map that keeps its height while empty.
fn line(color: Color) -> impl Bundle {
    let tall = Node {
        min_height: px(LINE_HEIGHT),
        ..default()
    };
    (
        text("", theme::FONT_SMALL, color),
        TextLayout::no_wrap(),
        tall,
    )
}

/// The lines along the bottom of the map: the progress of the route over the course line and
/// the place under the pointer.
fn foot() -> impl Bundle {
    let lines = children![
        (Action, line(theme::TEXT)),
        (Status, line(theme::TEXT_2)),
        crate::map::coords()
    ];
    (stack(), lines)
}

/// What is written over the map: the path of the levels, then the caption, in the top left
/// corner with north at the right end, and along the bottom the progress of the route over the
/// course line and the place under the pointer, with the scale at the right end. Each line
/// keeps its height while empty, so the others do not jump.
fn words() -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::all(px(MARGIN)),
        ..default()
    };
    let head = children![super::header::top(), crate::map::caption()];
    (
        node,
        children![
            ends(
                AlignItems::FlexStart,
                (stack(), head),
                super::legend::north()
            ),
            ends(AlignItems::FlexEnd, foot(), super::legend::scale()),
        ],
    )
}

/// The panel of the map, translucent like the side panel so the blurred game shows through: the
/// square map in its middle, the veil that fades one level into the next, and the words over
/// them, inside the padding of the panel.
fn area() -> impl Bundle {
    (
        Area,
        RelativeCursorPosition::default(),
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(theme::PANEL_BLUR),
        children![field(), crate::map::veil(), words()],
    )
}

/// Spawns the screen over the whole window and asks for the rows of the open tab.
pub(super) fn spawn(mut commands: Commands, mut board: ResMut<super::board::Board>) {
    let tab = board.tab;
    board.show(tab);
    let node = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        column_gap: px(BETWEEN),
        padding: UiRect::all(px(EDGE)),
        ..default()
    };
    commands.spawn((
        DespawnOnExit(Computer::Open),
        (Panel, Overlay, node),
        BackgroundColor(theme::SCRIM),
        Blurred,
        children![area(), side()],
    ));
}
