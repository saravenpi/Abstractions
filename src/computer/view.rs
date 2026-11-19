//! Static layout of the map screen: the map over the whole window but a slim panel on the
//! right, with where the player is written over its top left corner and the action line of
//! the course over its bottom. The panel holds the search box with its results, the
//! favourites and the players.
//!
//! The game keeps running under the screen. Every button of the screen is for the pointer
//! only: hovering one lights it and never moves the keyboard focus, which stays on the
//! selected result.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::layout::{LINE_HEIGHT, MARGIN, SIDE};
use super::map::field;
use super::open::Computer;
use crate::ui::{Overlay, text, theme};

/// Background of the side panel: a tone lighter than the map, which sets it apart with no line.
pub const PANEL: Color = Color::srgb(0.09, 0.09, 0.09);

/// What the search box says while nothing is typed.
pub(super) const SEARCH_HINT: &str = "Search (/)";

/// Marks the whole screen.
#[derive(Component)]
pub(super) struct Panel;

/// Marks the room of the map beside the panel, which cuts the chart at its edges.
#[derive(Component)]
pub struct Area;

/// A result button of the pool, counted from the top of the list.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Slot(pub usize);

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

/// Marks the line that says where the player is.
#[derive(Component)]
pub(super) struct Crumbs;

/// Marks the line that names the results and counts them.
#[derive(Component)]
pub(super) struct Summary;

/// Marks the line that shows the course, its distance and the route state.
#[derive(Component)]
pub(super) struct Status;

/// Marks the words of the action line of the course, shown while it has no button.
#[derive(Component)]
pub(super) struct Action;

/// Marks the room of the button of the action line of the course.
#[derive(Component)]
pub(super) struct ActionSlot;

/// Marks the search box, which a click makes the place typing goes to.
#[derive(Component)]
pub struct SearchBox;

/// Marks the text of the search box.
#[derive(Component)]
pub(super) struct SearchText;

/// The slim panel on the right: the search with its card, the favourites and the players.
fn side() -> impl Bundle {
    let node = Node {
        width: px(SIDE),
        height: percent(100),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::GAP_M),
        padding: UiRect::all(px(MARGIN)),
        overflow: Overflow::clip(),
        ..default()
    };
    (
        node,
        BackgroundColor(PANEL),
        children![
            super::results::search(),
            super::readout::card(),
            crate::map::favourites(),
            crate::map::players()
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

/// The lines along the bottom of the map: the action line of the course, which holds its
/// button or its words, over the course line and the place under the pointer.
fn foot() -> impl Bundle {
    let act = (
        Node {
            min_height: px(theme::ROW_COMPACT),
            align_items: AlignItems::Center,
            ..default()
        },
        children![(ActionSlot, Node::default()), (Action, line(theme::TEXT))],
    );
    let lines = children![act, (Status, line(theme::MUTED)), crate::map::coords()];
    (stack(), lines)
}

/// What is written over the map: the Back button and the path of the level shown, then where
/// the player is, in the top left corner with north at
/// the right end, and along the bottom the action line of the course over the course line
/// and the place under the pointer, with the scale at the right end. Each line keeps its
/// height while empty, so the others do not jump.
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
    let head = children![
        crate::map::path(),
        (Crumbs, line(theme::MUTED)),
        crate::map::caption()
    ];
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

/// The room of the map: the square map in its middle, the veil that fades one level into the
/// next, and the words over them.
fn area() -> impl Bundle {
    (
        Area,
        RelativeCursorPosition::default(),
        Node {
            flex_grow: 1.0,
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
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
        ..default()
    };
    commands.spawn((
        DespawnOnExit(Computer::Open),
        (Panel, Overlay, node),
        BackgroundColor(theme::BACKGROUND),
        children![area(), side()],
    ));
}
