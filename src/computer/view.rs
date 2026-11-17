//! Static layout of the computer screen: tabs, a fixed pool of rows, the route line and actions.

use bevy::prelude::*;

use super::board::{Board, Tab, VISIBLE_ROWS};
use super::open::Computer;
use crate::ui::{Tone, caps, column, footer, label, link, root, row_with, text, theme};

/// Width of the table in pixels.
const TABLE_WIDTH: f32 = theme::MENU_WIDTH * 2.4;

/// Key hints shown at the bottom of the screen.
const HINT: &str = "Up/Down select, Left/Right tab, Enter set destination, G engage, Tab close";

/// A row button of the pool, counted from the top of the screen.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Slot(pub usize);

/// Which text of a row a cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The name on the left.
    Title,
    /// The facts on the right.
    Detail,
}

/// A text of the row in this slot.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Cell(pub usize, pub Part);

/// A button that opens this tab.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct TabLink(pub Tab);

/// A button that acts on the destination.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Act {
    /// Hand the destination to the autopilot.
    Engage,
    /// Forget the destination.
    Clear,
}

/// Marks the line that names the table and counts its rows.
#[derive(Component)]
pub(super) struct Summary;

/// Marks the line that shows the destination and the route state.
#[derive(Component)]
pub(super) struct Status;

/// A horizontal strip of small buttons.
fn strip() -> Node {
    Node {
        column_gap: px(theme::SPACE),
        margin: UiRect::vertical(px(theme::SPACE)),
        ..default()
    }
}

/// One empty row of the pool. `show::rows` writes its texts.
fn row(slot: usize) -> impl Bundle {
    let title = (
        label("", theme::FONT_BODY, Tone::Plain),
        Cell(slot, Part::Title),
    );
    let detail = (
        label("", theme::FONT_SMALL, Tone::Muted),
        Cell(slot, Part::Detail),
    );
    (row_with(0, title, detail), Slot(slot))
}

/// The buttons that open each tab.
fn tabs() -> impl Bundle {
    let tab = |tab: Tab| (link(1, tab.label(), Tone::Muted), TabLink(tab));
    (
        strip(),
        Children::spawn(SpawnIter(Tab::ALL.into_iter().map(tab))),
    )
}

/// The buttons that engage or clear the destination.
fn actions() -> impl Bundle {
    (
        strip(),
        children![
            (link(2, "Engage", Tone::Accent), Act::Engage),
            (link(2, "Clear destination", Tone::Muted), Act::Clear),
        ],
    )
}

/// Spawns the screen over the dimmed world and asks for the rows of the open tab.
pub(super) fn spawn(mut commands: Commands, mut board: ResMut<Board>) {
    let tab = board.tab;
    board.show(tab);
    commands.spawn((
        DespawnOnExit(Computer::Open),
        root(),
        children![
            caps("Board computer"),
            tabs(),
            (Summary, text("", theme::FONT_SMALL, theme::MUTED)),
            (
                column(px(TABLE_WIDTH), 0.0),
                Children::spawn(SpawnIter((0..VISIBLE_ROWS).map(row))),
            ),
            (
                Status,
                text("", theme::FONT_SMALL, theme::TEXT),
                Node {
                    margin: UiRect::top(px(theme::SPACE * 2.0)),
                    ..default()
                },
            ),
            actions(),
            footer(HINT),
        ],
    ));
}
