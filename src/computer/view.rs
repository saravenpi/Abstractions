//! Static layout of the computer screen: tabs, a fixed pool of table rows beside the map and
//! the readout, the route line and the actions with their keys.

use bevy::prelude::*;

use super::board::{Board, Tab, VISIBLE_ROWS};
use super::open::Computer;
use super::readout::pane;
use crate::menus::key_name;
use crate::settings::{Persistent, Settings};
use crate::ui::{
    Icon, Selected, Tone, action, cell, column, footer, head, heading, icon, label, link, panel,
    root, strip, table_head, table_row, text, theme,
};

/// Widths in pixels of the mark, name, distance and facts columns.
const WIDTHS: [f32; 4] = [20.0, 190.0, 100.0, 360.0];

/// Key hints shown at the bottom of the screen.
const HINT: &str = "Up and down select. Left and right change tab. Enter sets the destination. \
                    On the map the wheel zooms, a drag pans and Backspace shows it all";

/// A row button of the pool, counted from the top of the screen.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Slot(pub usize);

/// Which text of a row a cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The icon that marks the destination or a visited place.
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

/// A button that acts on the destination or on the screen.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Act {
    /// Hand the destination to the autopilot.
    Engage,
    /// Forget the destination.
    Clear,
    /// Close the screen.
    Close,
}

/// Marks the line that names the table and counts its rows.
#[derive(Component)]
pub(super) struct Summary;

/// Marks the line that shows the destination and the route state.
#[derive(Component)]
pub(super) struct Status;

/// One empty row of the pool. `show::rows` writes its texts.
fn row(slot: usize) -> impl Bundle {
    let small =
        move |part: Part, tone: Tone| (label("", theme::FONT_SMALL, tone), Cell(slot, part));
    let mark = (icon(Icon::Check, Tone::Muted), Cell(slot, Part::Mark));
    let title = (
        label("", theme::FONT_BODY, Tone::Plain),
        Cell(slot, Part::Title),
    );
    (
        table_row(
            0,
            theme::ROW_COMPACT,
            children![
                cell(WIDTHS[0], mark),
                cell(WIDTHS[1], title),
                cell(WIDTHS[2], small(Part::Distance, Tone::Plain)),
                cell(WIDTHS[3], small(Part::Detail, Tone::Muted)),
            ],
        ),
        Slot(slot),
        Selected(false),
    )
}

/// Returns the icon shown on a tab: a planet for the system, a star for the galaxy of stars.
/// The icon pack has no galaxy, so the universe tab has a label only.
fn tab_icon(tab: Tab) -> Option<Icon> {
    match tab {
        Tab::System => Some(Icon::Earth),
        Tab::Galaxy => Some(Icon::Star),
        Tab::Universe => None,
    }
}

/// The buttons that open each tab, with their icons. The open one is selected.
fn tabs() -> impl Bundle {
    let tab = |tab: Tab| {
        (
            link(1, tab_icon(tab), tab.label(), Tone::Muted),
            TabLink(tab),
            Selected(false),
        )
    };
    (
        strip(),
        Children::spawn(SpawnIter(Tab::ALL.into_iter().map(tab))),
    )
}

/// The column names and the pool of rows.
fn table() -> impl Bundle {
    (
        column(px(WIDTHS.iter().sum::<f32>() + theme::GAP_M * 4.0), 0.0),
        Children::spawn((
            Spawn(table_head(children![
                head(WIDTHS[0], ""),
                head(WIDTHS[1], "Name"),
                head(WIDTHS[2], "Distance"),
                head(WIDTHS[3], "Facts"),
            ])),
            SpawnIter((0..VISIBLE_ROWS).map(row)),
        )),
    )
}

/// The table of the open tab beside its map and the readout of the selected row.
fn charted() -> impl Bundle {
    (
        Node {
            column_gap: px(theme::GAP_M),
            align_items: AlignItems::FlexStart,
            ..default()
        },
        children![table(), pane()],
    )
}

/// The actions on the destination and the screen, each with its icon and its key.
fn actions(engage: &str, close: &str) -> impl Bundle {
    (
        strip(),
        children![
            (
                action(Icon::Send, "Engage", engage, Tone::Accent),
                Act::Engage
            ),
            (
                action(Icon::Eraser, "Clear destination", "Del", Tone::Plain),
                Act::Clear
            ),
            (action(Icon::Close, "Close", close, Tone::Muted), Act::Close),
        ],
    )
}

/// Spawns the screen over the blurred game and asks for the rows of the open tab.
pub(super) fn spawn(
    mut commands: Commands,
    mut board: ResMut<Board>,
    settings: Res<Persistent<Settings>>,
) {
    let tab = board.tab;
    board.show(tab);
    let keys = &settings.controls.keys;
    let close = format!("{} or Esc", key_name(keys.computer));
    commands.spawn((
        DespawnOnExit(Computer::Open),
        root(),
        children![
            (
                panel(),
                children![
                    heading("Board computer"),
                    tabs(),
                    (Summary, text("", theme::FONT_SMALL, theme::MUTED)),
                    charted(),
                    (
                        Status,
                        text("", theme::FONT_SMALL, theme::TEXT),
                        Node {
                            margin: UiRect::top(px(theme::GAP_S)),
                            ..default()
                        },
                    ),
                    actions(&key_name(keys.engage), &close),
                ]
            ),
            footer(HINT),
        ],
    ));
}
