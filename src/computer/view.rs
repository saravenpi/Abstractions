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

use super::layout::{MARGIN, SIDE};
use super::map::field;
use super::open::Computer;
use crate::ui::{Overlay, Tone, column, hairline, key_cap, label, text, theme};

/// Key hints shown at the bottom of the panel, one short line each.
pub(super) const HINT: &str = "Wheel zoom, drag pan\n0 centre, B waypoint";

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

/// Marks the words of the action line of the course.
#[derive(Component)]
pub(super) struct Action;

/// Marks the key of the action line of the course.
#[derive(Component)]
pub(super) struct ActionKey;

/// Marks the search box, which a click makes the place typing goes to.
#[derive(Component)]
pub struct SearchBox;

/// Marks the text of the search box.
#[derive(Component)]
pub(super) struct SearchText;

/// The slim panel on the right: the search, the favourites, the players and the key hints.
fn side() -> impl Bundle {
    let node = Node {
        width: px(SIDE),
        height: percent(100),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::GAP_M),
        padding: UiRect::all(px(MARGIN)),
        border: UiRect::left(px(theme::LINE)),
        overflow: Overflow::clip(),
        ..default()
    };
    let hint = (
        text(HINT, theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    );
    (
        node,
        hairline(),
        BackgroundColor(theme::SCRIM),
        children![
            super::results::search(),
            super::readout::card(),
            crate::map::favourites(),
            crate::map::players(),
            hint
        ],
    )
}

/// What is written over the map: where the player is in the top left corner, and the action
/// line of the course over the course line along the bottom, with the place under the
/// pointer.
fn words() -> impl Bundle {
    let line = |color: Color| (text("", theme::FONT_SMALL, color), TextLayout::no_wrap());
    let node = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::all(px(MARGIN)),
        ..default()
    };
    let act = (
        Node {
            column_gap: px(theme::GAP_M),
            ..default()
        },
        children![
            (ActionKey, key_cap("")),
            (Action, label("", theme::FONT_BODY, Tone::Plain))
        ],
    );
    let foot = (
        column(percent(100), theme::GAP_XS),
        children![act, (Status, line(theme::MUTED)), crate::map::coords()],
    );
    let head = (
        column(percent(100), theme::GAP_XS),
        children![(Crumbs, line(theme::TEXT)), crate::map::caption()],
    );
    (node, children![head, foot])
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
