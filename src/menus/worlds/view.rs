//! Static layout of the worlds menu: the table, the bars of actions and the empty state.

use bevy::prelude::*;

use super::rows::{Cell, Part, Row};
use super::{Mode, Notice, VISIBLE_ROWS, Worlds, bars, ops};
use crate::menus::{Menu, Nav};
use crate::saves::SaveDir;
use crate::ui::{
    Cancels, Icon, Selected, Tone, back, cell, column, empty_state, head, hints, item, label,
    legend, notice, page, page_body, page_foot, page_head, page_status, page_title, table_head,
    table_row, theme,
};

/// Columns of the alignment grid the world, seed, play time and last seen columns take.
const SPANS: [u32; 4] = [4, 3, 2, 2];

/// Key hints shown in the footer, after the back action.
const HINTS: [(&str, &str); 3] = [
    ("Select", "Up Down"),
    ("Play", "Enter"),
    ("Choose while deleting", "Left Right"),
];

/// Why the list is empty, and what to do about it.
const WHY: &str = "Every world is a whole universe grown from one seed. Create one to wake up on your home planet.";

/// Returns the width in pixels of column `at` of the table.
fn width(at: usize) -> f32 {
    theme::span(SPANS[at])
}

/// Marks the line that counts the worlds and names their order.
#[derive(Component)]
pub(super) struct Summary;

/// Marks the line that shows a refused name or a failed file operation.
#[derive(Component)]
pub(super) struct Hint;

/// Marks the invitation shown when there is no world.
#[derive(Component)]
pub(super) struct Empty;

/// Marks the table of worlds, column names included, hidden when there is no world.
#[derive(Component)]
pub(super) struct Listing;

/// Marks the button of the invitation, which creates a world.
#[derive(Component)]
pub(super) struct Invite;

/// One empty row of the pool. `rows::show` writes its texts.
fn row(slot: usize) -> impl Bundle {
    let small = move |part: Part| (label("", theme::FONT_SMALL, Tone::Muted), Cell(slot, part));
    let name = (
        column(percent(100), 0.0),
        children![
            (
                label("", theme::FONT_BODY, Tone::Plain),
                Cell(slot, Part::Name)
            ),
            small(Part::Place)
        ],
    );
    let cells = children![
        cell(width(0), name),
        cell(width(1), small(Part::Seed)),
        cell(width(2), small(Part::Played)),
        cell(width(3), small(Part::Last)),
    ];
    (
        table_row(0, theme::ROW_TALL, cells),
        Row(slot),
        Selected(false),
    )
}

/// The column names and the pool of rows, as wide as its columns, their gutters and the inset
/// of a row on both sides.
fn table() -> impl Bundle {
    let all: u32 = SPANS.iter().sum();
    (
        Listing,
        column(px(theme::span(all) + 2.0 * theme::INSET), 0.0),
        Children::spawn((
            Spawn(table_head(children![
                head(width(0), "World"),
                head(width(1), "Seed"),
                head(width(2), "Played"),
                head(width(3), "Last seen"),
            ])),
            SpawnIter((0..VISIBLE_ROWS).map(row)),
        )),
    )
}

/// The empty state shown when there is no world yet, and the action that makes the first one.
fn empty() -> impl Bundle {
    (
        Empty,
        column(px(theme::span(6)), theme::S3),
        children![
            empty_state(Icon::Planet, "No worlds yet", WHY),
            (
                item(Icon::Plus, "Create a world", Tone::Accent),
                Nav::To(Menu::NewGame),
                Invite
            ),
        ],
    )
}

/// Loads the worlds and spawns the menu. The rows are filled by `rows::show`.
pub(super) fn spawn(mut commands: Commands, dir: Res<SaveDir>, mut worlds: ResMut<Worlds>) {
    worlds.window = default();
    worlds.fill(ops::load(&dir.0), None);
    commands.insert_resource(Mode::Browse);
    commands.insert_resource(Notice::default());
    let head = page_head(children![page_title("Worlds"), (Summary, page_status(""))]);
    let body = (
        page_body(FlexDirection::Column),
        children![table(), empty(), (Hint, notice()), bars::bars()],
    );
    let foot = page_foot(children![
        (back("Back"), Nav::Back, Cancels),
        legend(hints(&HINTS))
    ]);
    commands.spawn((DespawnOnExit(Menu::Worlds), page(head, body, foot)));
}
