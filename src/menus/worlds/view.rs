//! Static layout of the worlds menu: the table, the bars of actions and the empty state.

use bevy::prelude::*;

use super::rows::{Cell, Part, Row};
use super::{Mode, Notice, VISIBLE_ROWS, Worlds, bars, ops};
use crate::menus::{Menu, Nav};
use crate::saves::SaveDir;
use crate::ui::{
    Icon, Selected, Tone, cell, column, footer, head, heading, item, label, notice, panel, root,
    table_head, table_row, text, theme,
};

/// Widths in pixels of the world, seed, play time and last played columns.
const WIDTHS: [f32; 4] = [270.0, 165.0, 110.0, 160.0];

/// Height of one world row in pixels: a name with the place under it.
const ROW: f32 = 44.0;

/// Key hints shown at the bottom of the screen.
const HINT: &str =
    "Up and down select. Enter plays. While deleting, left and right choose and Enter applies";

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
    (
        table_row(
            0,
            ROW,
            children![
                cell(WIDTHS[0], name),
                cell(WIDTHS[1], small(Part::Seed)),
                cell(WIDTHS[2], small(Part::Played)),
                cell(WIDTHS[3], small(Part::Last)),
            ],
        ),
        Row(slot),
        Selected(false),
    )
}

/// The column names and the pool of rows.
fn table() -> impl Bundle {
    (
        Listing,
        column(px(WIDTHS.iter().sum::<f32>() + theme::GAP_M * 4.0), 0.0),
        Children::spawn((
            Spawn(table_head(children![
                head(WIDTHS[0], "World"),
                head(WIDTHS[1], "Seed"),
                head(WIDTHS[2], "Played"),
                head(WIDTHS[3], "Last played"),
            ])),
            SpawnIter((0..VISIBLE_ROWS).map(row)),
        )),
    )
}

/// The invitation shown when there is no world yet.
fn empty() -> impl Bundle {
    (
        Empty,
        column(px(theme::MENU_WIDTH * 1.4), theme::GAP_S),
        children![
            text("No worlds yet", theme::FONT_HEADING, theme::TEXT),
            text(
                "Every world is a whole universe grown from one seed. Create one to wake up on your home planet.",
                theme::FONT_SMALL,
                theme::TEXT_2
            ),
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
    commands.spawn((
        DespawnOnExit(Menu::Worlds),
        root(),
        children![
            (
                panel(),
                children![
                    heading("Worlds"),
                    table(),
                    empty(),
                    (Summary, text("", theme::FONT_SMALL, theme::TEXT_2)),
                    (Hint, notice()),
                    bars::bars(),
                ]
            ),
            footer(HINT),
        ],
    ));
}
