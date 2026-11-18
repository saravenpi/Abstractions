//! Static layout of the computer panel on the right edge of the window: the header with the
//! breadcrumb and the tabs, the map, a fixed pool of list rows, the card of the selected row
//! and the footer with the course, the actions and the key hints.
//!
//! The game stays visible and keeps running to the left of the panel.

use bevy::prelude::*;

use super::board::VISIBLE_ROWS;
use super::layout::{PAD, SMALLEST, panel_width};
use super::map::field;
use super::open::Computer;
use super::readout::card;
use super::tab::Tab;
use crate::menus::key_name;
use crate::settings::{Persistent, Settings};
use crate::ui::{
    Icon, Overlay, Selected, Tone, action, cell, column, icon, label, link, table_row, text, theme,
};

/// Widths in pixels of the icon, name, distance and facts columns of a row.
const WIDTHS: [f32; 4] = [24.0, 130.0, 76.0, 200.0];

/// Key hints shown at the bottom of the panel.
const HINT: &str = "Type to filter. F2 sorts. Map: wheel, drag, 0";

/// Marks the panel, which the window sizes.
#[derive(Component)]
pub(super) struct Panel;

/// A row button of the pool, counted from the top of the list.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Slot(pub usize);

/// Which text of a row a cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The icon that says what the row is, or marks the destination or the place of the ship.
    Mark,
    /// The name.
    Title,
    /// The distance from the ship.
    Distance,
    /// The other facts.
    Detail,
}

/// A text of the row in this slot.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Cell(pub usize, pub Part);

/// A button that opens this tab.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct TabLink(pub Tab);

/// A button of the footer that acts on the course.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Set the course to the row under the cursor.
    Set,
    /// Hand the course to the autopilot.
    Engage,
    /// Forget the course.
    Clear,
}

/// Marks the line that says where the ship is.
#[derive(Component)]
pub(super) struct Crumbs;

/// Marks the line that names the list and counts its rows.
#[derive(Component)]
pub(super) struct Summary;

/// Marks the line that shows the course and the route state.
#[derive(Component)]
pub(super) struct Status;

/// One empty row of the pool. `show::rows` writes its texts.
fn row(slot: usize) -> impl Bundle {
    let small =
        move |part: Part, tone: Tone| (label("", theme::FONT_SMALL, tone), Cell(slot, part));
    let mark = (icon(Icon::Check, Tone::Muted), Cell(slot, Part::Mark));
    (
        table_row(
            0,
            theme::ROW_COMPACT,
            children![
                cell(WIDTHS[0], mark),
                cell(WIDTHS[1], small(Part::Title, Tone::Plain)),
                cell(WIDTHS[2], small(Part::Distance, Tone::Plain)),
                cell(WIDTHS[3], small(Part::Detail, Tone::Muted)),
            ],
        ),
        Slot(slot),
        Selected(false),
    )
}

/// Returns the icon shown on a tab: a planet for the system, a star for the galaxy of stars
/// and a map for the universe, since the icon pack has no galaxy.
fn tab_icon(tab: Tab) -> Icon {
    match tab {
        Tab::System => Icon::Earth,
        Tab::Galaxy => Icon::Star,
        Tab::Universe => Icon::Map,
    }
}

/// The header: where the ship is, the tabs with their icons, and the line that counts the rows.
fn header() -> impl Bundle {
    let tab = |tab: Tab| {
        let button = link(1, tab_icon(tab), tab.label(), Tone::Muted);
        (button, TabLink(tab), Selected(false))
    };
    let tabs = (
        Node {
            column_gap: px(theme::GAP_S),
            ..default()
        },
        Children::spawn(SpawnIter(Tab::ALL.into_iter().map(tab))),
    );
    (
        column(percent(100), theme::GAP_XS),
        children![
            (Crumbs, text("", theme::FONT_SMALL, theme::TEXT)),
            tabs,
            (Summary, text("", theme::FONT_SMALL, theme::MUTED)),
        ],
    )
}

/// The pool of rows of the list. Only the rows that fit the window are shown.
fn list() -> impl Bundle {
    (
        column(percent(100), 0.0),
        Children::spawn(SpawnIter((0..VISIBLE_ROWS).map(row))),
    )
}

/// The footer: the course, the actions on it with their keys, and the key hints.
fn footer(engage: &str) -> impl Bundle {
    let set = action(Icon::MapPin, "Set course", "Enter", Tone::Accent);
    let go = action(Icon::Send, "Engage autopilot", engage, Tone::Accent);
    let clear = action(Icon::Eraser, "Clear course", "Del", Tone::Plain);
    let actions = (
        Node {
            column_gap: px(theme::GAP_S),
            height: px(theme::ROW_HEIGHT),
            ..default()
        },
        children![(set, Act::Set), (go, Act::Engage), (clear, Act::Clear)],
    );
    (
        column(percent(100), theme::GAP_XS),
        children![
            (Status, text("", theme::FONT_SMALL, theme::TEXT)),
            actions,
            (
                text(HINT, theme::FONT_SMALL, theme::MUTED),
                TextLayout::no_wrap()
            ),
        ],
    )
}

/// Spawns the panel on the right edge of the window and asks for the rows of the open tab.
pub(super) fn spawn(
    mut commands: Commands,
    mut board: ResMut<super::board::Board>,
    settings: Res<Persistent<Settings>>,
) {
    let tab = board.tab;
    board.show(tab);
    let node = Node {
        position_type: PositionType::Absolute,
        top: px(0),
        right: px(0),
        bottom: px(0),
        width: px(panel_width(SMALLEST.x)),
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::GAP_S),
        padding: UiRect::all(px(PAD)),
        border: UiRect::left(px(theme::LINE)),
        overflow: Overflow::clip(),
        ..default()
    };
    let engage = key_name(settings.controls.keys.engage);
    commands.spawn((
        DespawnOnExit(Computer::Open),
        (Panel, Overlay, node),
        BackgroundColor(theme::SCRIM),
        BorderColor::all(theme::HAIRLINE),
        children![header(), field(), list(), card(), footer(&engage)],
    ));
}
