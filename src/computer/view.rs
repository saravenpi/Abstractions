//! Static layout of the computer panel on the right edge of the window: the header with the
//! breadcrumb and the tabs, the map, the list with its title over a fixed pool of rows, the
//! card of the selected row and the footer with the course, the main button and the key hints.
//!
//! The game stays visible and keeps running to the left of the panel. Every button of the
//! panel is for the pointer only: hovering one lights it and never moves the keyboard focus,
//! which stays on the selected row.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;

use super::board::VISIBLE_ROWS;
use super::layout::{BUTTON, ROW, SMALLEST, measure};
use super::map::field;
use super::open::Computer;
use super::primary::{Spare, button};
use super::readout::card;
use super::tab::Tab;
use crate::ui::{
    Blurred, Icon, Look, Overlay, Selected, Tone, action, cell, column, entry, frame, hairline,
    icon, label, text, theme,
};

/// Widths in pixels of the icon, name and distance columns of a row. The facts take the rest.
const WIDTHS: [f32; 3] = [24.0, 140.0, 80.0];

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

/// What a button of the footer does to the course.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Set the course to the selected row.
    Set,
    /// Hand the course to the autopilot.
    Engage,
    /// Forget the course.
    Clear,
    /// Set the course to the next place on the way home.
    Home,
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

/// The node of one row of the list: a strip closed by a hairline, its columns lined up.
fn strip() -> Node {
    Node {
        width: percent(100),
        height: px(ROW),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_M - theme::GAP_XS),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        border: UiRect::bottom(px(theme::LINE)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// One empty row of the pool, a button for the pointer only whose look `hover::light` sets.
/// `show::rows` writes its texts.
fn row(slot: usize) -> impl Bundle {
    let small =
        move |part: Part, tone: Tone| (label("", theme::FONT_SMALL, tone), Cell(slot, part));
    let mark = (icon(Icon::Check, Tone::Muted), Cell(slot, Part::Mark));
    let facts = Node {
        flex_grow: 1.0,
        min_width: px(0),
        overflow: Overflow::clip(),
        ..default()
    };
    (
        (Button, RelativeCursorPosition::default(), Look::default()),
        (strip(), hairline(), BackgroundColor(Color::NONE)),
        children![
            cell(WIDTHS[0], mark),
            cell(WIDTHS[1], small(Part::Title, Tone::Plain)),
            cell(WIDTHS[2], small(Part::Distance, Tone::Plain)),
            (facts, children![small(Part::Detail, Tone::Muted)]),
        ],
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

/// The header: where the ship is, muted, over the tabs with their icons.
fn header() -> impl Bundle {
    let tab = |tab: Tab| {
        let node = Node {
            height: px(theme::ROW_HEIGHT),
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(theme::GAP_S)),
            ..default()
        };
        let words = entry(tab_icon(tab), tab.label(), theme::FONT_SMALL, Tone::Muted);
        (
            frame(),
            node,
            children![words],
            TabLink(tab),
            Selected(false),
        )
    };
    let tabs = (
        Node {
            column_gap: px(theme::GAP_S),
            ..default()
        },
        Children::spawn(SpawnIter(Tab::ALL.into_iter().map(tab))),
    );
    let crumbs = text("", theme::FONT_SMALL, theme::MUTED);
    (
        column(percent(100), theme::GAP_XS),
        children![(Crumbs, crumbs, TextLayout::no_wrap()), tabs],
    )
}

/// The list: its title, which counts the rows, over the pool of rows. Only the rows that fit
/// the window are shown, and the list takes the height the other sections leave.
fn list() -> impl Bundle {
    let rows = (
        column(percent(100), 0.0),
        Children::spawn(SpawnIter((0..VISIBLE_ROWS).map(row))),
    );
    let title = text("", theme::FONT_SMALL, theme::MUTED);
    (
        Node {
            flex_grow: 1.0,
            min_height: px(0),
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(theme::GAP_XS),
            overflow: Overflow::clip(),
            ..default()
        },
        children![(Summary, title, TextLayout::no_wrap()), rows],
    )
}

/// The footer: the course, the main button with the small ones that set the course home and
/// clear it beside it, and the key hints.
fn footer() -> impl Bundle {
    let spare = (
        action(Icon::Eraser, "", "Del", Tone::Plain),
        Act::Clear,
        Spare,
    );
    let home = (action(Icon::Home, "Course", "Home", Tone::Plain), Act::Home);
    let actions = (
        Node {
            column_gap: px(theme::GAP_S),
            height: px(BUTTON),
            ..default()
        },
        children![button(), home, spare],
    );
    let line = |words: &'static str, color: Color| {
        (text(words, theme::FONT_SMALL, color), TextLayout::no_wrap())
    };
    (
        column(percent(100), theme::GAP_XS),
        children![
            (Status, line("", theme::TEXT)),
            actions,
            line(HINT, theme::MUTED),
        ],
    )
}

/// Spawns the panel on the right edge of the window and asks for the rows of the open tab.
pub(super) fn spawn(mut commands: Commands, mut board: ResMut<super::board::Board>) {
    let tab = board.tab;
    board.show(tab);
    let fit = measure(SMALLEST);
    let node = Node {
        position_type: PositionType::Absolute,
        top: px(0),
        right: px(0),
        bottom: px(0),
        width: px(fit.panel),
        flex_direction: FlexDirection::Column,
        row_gap: px(fit.margin),
        padding: UiRect::all(px(fit.margin)),
        border: UiRect::left(px(theme::LINE)),
        overflow: Overflow::clip(),
        ..default()
    };
    commands.spawn((
        DespawnOnExit(Computer::Open),
        (Panel, Overlay, Blurred, node),
        BackgroundColor(theme::SCRIM),
        hairline(),
        children![header(), field(), list(), card(), footer()],
    ));
}
