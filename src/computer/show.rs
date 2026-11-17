//! Filling the computer screen: building the rows and writing every text.

use bevy::prelude::*;

use super::board::{Board, Tab, VISIBLE_ROWS};
use super::open::ready;
use super::route::{Destination, RouteStatus};
use super::scene::Scene;
use super::view::{Cell, Part, Slot, Status, Summary, TabLink};
use crate::ui::Tone;

/// Seconds between two refreshes of the system tab, whose distances change while flying.
const REFRESH: f32 = 0.5;

/// Builds the rows when the tab asks for them, when the chart of known systems is finished,
/// and twice a second on the system tab.
fn rebuild(time: Res<Time>, scene: Scene, mut board: ResMut<Board>, mut since: Local<f32>) {
    *since += time.delta_secs();
    let moving = board.tab == Tab::System && *since >= REFRESH;
    if !(board.stale || moving || scene.charted()) {
        return;
    }
    *since = 0.0;
    let (heading, lines) = scene.table(board.tab);
    board.fill(heading, lines);
}

/// Writes the texts of the drawn rows, hides the rows past the end of the table, and gives the
/// accent to the row that is the destination.
fn rows(
    board: Res<Board>,
    destination: Res<Destination>,
    mut cells: Query<(&Cell, &mut Text, &mut Tone)>,
    mut slots: Query<(&Slot, &mut Node)>,
) {
    for (Cell(slot, part), mut text, mut tone) in &mut cells {
        let Some(line) = board.at(*slot) else {
            continue;
        };
        let chosen = destination.0 == Some(line.target);
        let (shown, wanted) = match (part, chosen) {
            (Part::Title, true) => (&line.title, Tone::Accent),
            (Part::Title, false) => (&line.title, Tone::Plain),
            (Part::Detail, _) => (&line.detail, Tone::Muted),
        };
        text.set_if_neq(Text(shown.clone()));
        tone.set_if_neq(wanted);
    }
    for (slot, mut node) in &mut slots {
        let display = match board.at(slot.0) {
            Some(_) => Display::Flex,
            None => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
    }
}

/// Returns the line under the tabs: the table name and which rows of it are drawn.
fn summary(board: &Board) -> String {
    let (first, count) = (board.top + 1, board.lines.len());
    let last = (board.top + VISIBLE_ROWS).min(count);
    match count {
        0 => format!("{} - nothing charted yet", board.heading),
        _ => format!("{} - {first} to {last} of {count}", board.heading),
    }
}

/// Lights the open tab and writes the summary line.
fn tabs(
    board: Res<Board>,
    links: Query<&TabLink>,
    mut labels: Query<(&ChildOf, &mut Tone)>,
    mut lines: Query<&mut Text, With<Summary>>,
) {
    for (parent, mut tone) in &mut labels {
        if let Ok(TabLink(tab)) = links.get(parent.parent()) {
            let wanted = if *tab == board.tab {
                Tone::Accent
            } else {
                Tone::Muted
            };
            tone.set_if_neq(wanted);
        }
    }
    for mut text in &mut lines {
        text.set_if_neq(Text(summary(&board)));
    }
}

/// Writes the destination, its distance and the state of the route.
fn status(
    scene: Scene,
    destination: Res<Destination>,
    route: Res<RouteStatus>,
    mut lines: Query<&mut Text, With<Status>>,
) {
    let shown = scene.route(destination.0, *route);
    for mut text in &mut lines {
        text.set_if_neq(Text(shown.clone()));
    }
}

/// Registers the systems that fill the screen.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (rebuild, rows, tabs, status)
            .chain()
            .after(super::Input)
            .run_if(ready),
    );
}
