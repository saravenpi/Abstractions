//! Layout of the multiplayer menu: the list with its two sections, the form, the notice and
//! the bars. The rows are built again whenever the number of servers changes.

use bevy::prelude::*;

use super::form_view::block;
use super::rows::{Cell, Part, Row, listed};
use super::{Ended, Form, FormKind, Mode, Notice, Pane, Servers, bars};
use crate::menus::Menu;
use crate::net::{Lobby, RefreshLobby};
use crate::settings::{Persistent, Settings};
use crate::ui::{
    Icon, Tone, caps, cell, column, footer, heading, icon, label, notice, panel, root, scroller,
    table_row, text, theme,
};

/// Widths in pixels of the name and the status columns.
const WIDTHS: [f32; 2] = [420.0, 150.0];

/// Width of the list and the form.
const WIDTH: f32 = theme::ICON + WIDTHS[0] + WIDTHS[1] + theme::GAP_M * 3.0;

/// Height of one server row in pixels: a name with the address under it.
const ROW: f32 = 44.0;

/// Most pixels the list takes before it scrolls.
const LIST_HEIGHT: f32 = 300.0;

/// Key hints shown at the bottom of the screen.
const HINT: &str = "Up and down select. Enter joins. Tab switches fields. Esc goes back";

/// Marks the column that holds the sections and their rows.
#[derive(Component)]
pub(super) struct List;

/// Marks the line that shows a refused field, a failed save or why a session ended.
#[derive(Component)]
pub(super) struct Hint;

/// One row: the icon of its section, the name with the address under it, and the status.
fn row(index: usize, symbol: Icon) -> impl Bundle {
    let small = move |part: Part| (label("", theme::FONT_SMALL, Tone::Muted), Cell(index, part));
    let name = (
        column(percent(100), 0.0),
        children![
            (
                label("", theme::FONT_BODY, Tone::Plain),
                Cell(index, Part::Name)
            ),
            small(Part::Address)
        ],
    );
    (
        table_row(
            0,
            ROW,
            children![
                cell(theme::ICON, icon(symbol, Tone::Plain)),
                cell(WIDTHS[0], name),
                cell(WIDTHS[1], small(Part::Status)),
            ],
        ),
        Row(index),
    )
}

/// The name of a section, with room above it.
fn caption(content: &str) -> impl Bundle {
    (
        caps(content),
        Node {
            margin: UiRect::new(px(0), px(0), px(theme::GAP_M), px(theme::GAP_XS)),
            ..default()
        },
    )
}

/// A muted line in place of an empty section.
fn empty(content: &str) -> impl Bundle {
    (
        text(content, theme::FONT_SMALL, theme::MUTED),
        Node {
            margin: UiRect::all(px(theme::GAP_S)),
            ..default()
        },
    )
}

/// The scrolling column of the sections, drawn while browsing.
fn list_pane() -> impl Bundle {
    (
        Pane::List,
        column(px(WIDTH), 0.0),
        children![(List, scroller(LIST_HEIGHT))],
    )
}

/// Spawns the sections of `servers` under `list`: the saved servers, then the network games.
fn fill(commands: &mut Commands, list: Entity, servers: &Servers) {
    let (saved, lan) = servers.shape();
    commands.spawn((caption("Your servers"), ChildOf(list)));
    for index in 0..saved {
        commands.spawn((row(index, Icon::Server), ChildOf(list)));
    }
    if saved == 0 {
        commands.spawn((empty("No saved servers. Add one with A"), ChildOf(list)));
    }
    commands.spawn((caption("On your network"), ChildOf(list)));
    for index in saved..saved + lan {
        commands.spawn((row(index, Icon::Wifi), ChildOf(list)));
    }
    if lan == 0 {
        commands.spawn((empty("Looking for games..."), ChildOf(list)));
    }
}

/// Builds the rows again when the list is new or the number of servers in a section changed.
pub(super) fn layout(
    mut commands: Commands,
    servers: Res<Servers>,
    lists: Query<Entity, With<List>>,
    fresh: Query<(), Added<List>>,
    mut shape: Local<(usize, usize)>,
) {
    if fresh.is_empty() && *shape == servers.shape() {
        return;
    }
    *shape = servers.shape();
    for list in &lists {
        commands.entity(list).despawn_children();
        fill(&mut commands, list, &servers);
    }
}

/// What the menu opens with.
type Opening<'w> = (Res<'w, Persistent<Settings>>, Res<'w, Lobby>);

/// Fills the list, asks every server for its status, and spawns the menu. Without a player
/// name the menu asks for one first. A session that just ended shows why.
pub(super) fn spawn(
    mut commands: Commands,
    (settings, lobby): Opening,
    mut servers: ResMut<Servers>,
    mut ended: ResMut<Ended>,
    mut refresh: MessageWriter<RefreshLobby>,
) {
    servers.cursor = 0;
    servers.fill(listed(&settings.servers, &lobby));
    let mode = match settings.player_name.trim().is_empty() {
        true => Mode::Form(Form::new(FormKind::Player, String::new(), String::new())),
        false => Mode::Browse,
    };
    commands.insert_resource(mode);
    commands.insert_resource(Notice(ended.0.take().unwrap_or_default()));
    refresh.write(RefreshLobby);
    commands.spawn((
        DespawnOnExit(Menu::Multiplayer),
        root(),
        children![
            (
                panel(),
                children![
                    heading("Multiplayer"),
                    list_pane(),
                    block(WIDTH),
                    (Hint, notice()),
                    bars::bars(),
                ]
            ),
            footer(HINT),
        ],
    ));
}

/// Writes the notice line.
pub(super) fn show_notice(notice: Res<Notice>, mut hints: Query<&mut Text, With<Hint>>) {
    if !notice.is_changed() {
        return;
    }
    for mut text in &mut hints {
        text.set_if_neq(Text(notice.0.clone()));
    }
}
