//! Filling the computer screen: building the rows and writing every text when it changes.

use bevy::prelude::*;

use super::board::{Board, Tab, VISIBLE_ROWS};
use super::format::distance;
use super::open::ready;
use super::route::{Destination, RouteStatus};
use super::rows::Line;
use super::scene::Scene;
use super::view::{Cell, Part, Slot, Status, Summary, TabLink};
use crate::ui::{Icon, Selected, Tone};

/// Seconds between two refreshes of what changes while flying: the distances of the system tab
/// and of the route line.
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
    board.ship = scene.ship();
    board.worlds = scene.worlds(&lines);
    board.fill(heading, lines);
}

/// Returns the text and the tone of one cell of `line`. The destination wears a pin, and a
/// visited place a check.
pub(super) fn cell_text(line: &Line, part: Part, chosen: bool) -> (String, Tone) {
    match (part, chosen, line.visited) {
        (Part::Mark, true, _) => (Icon::MapPin.glyph(), Tone::Accent),
        (Part::Mark, false, true) => (Icon::Check.glyph(), Tone::Muted),
        (Part::Mark, false, false) => (String::new(), Tone::Muted),
        (Part::Title, true, _) => (line.title.clone(), Tone::Accent),
        (Part::Title, false, _) => (line.title.clone(), Tone::Plain),
        (Part::Distance, _, _) => (distance(line.distance), Tone::Plain),
        (Part::Detail, _, _) => (line.facts().to_string(), Tone::Muted),
    }
}

/// What the rows are drawn from.
type Source<'w> = (Res<'w, Board>, Res<'w, Destination>);

/// Writes the texts of the drawn rows, hides the rows past the end of the table, and marks the
/// row under the cursor. Does nothing while neither the table nor the destination changed.
fn rows(
    (board, destination): Source,
    fresh: Query<(), Added<Slot>>,
    mut cells: Query<(&Cell, &mut Text, &mut Tone)>,
    mut slots: Query<(&Slot, &mut Node, &mut Selected)>,
) {
    if !(board.is_changed() || destination.is_changed() || !fresh.is_empty()) {
        return;
    }
    for (Cell(slot, part), mut text, mut tone) in &mut cells {
        let Some(line) = board.at(*slot) else {
            continue;
        };
        let (shown, wanted) = cell_text(line, *part, destination.0 == Some(line.target));
        text.set_if_neq(Text(shown));
        tone.set_if_neq(wanted);
    }
    for (slot, mut node, mut selected) in &mut slots {
        let display = match board.at(slot.0) {
            Some(_) => Display::Flex,
            None => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
        selected.set_if_neq(Selected(board.top + slot.0 == board.cursor));
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

/// Selects the open tab and writes the summary line, when the table changed.
fn tabs(
    board: Res<Board>,
    fresh: Query<(), Added<Summary>>,
    mut links: Query<(&TabLink, &mut Selected)>,
    mut lines: Query<&mut Text, With<Summary>>,
) {
    if !(board.is_changed() || !fresh.is_empty()) {
        return;
    }
    for (TabLink(tab), mut selected) in &mut links {
        selected.set_if_neq(Selected(*tab == board.tab));
    }
    for mut text in &mut lines {
        text.set_if_neq(Text(summary(&board)));
    }
}

/// What the route line is drawn from.
type Route<'w> = (Res<'w, Destination>, Res<'w, RouteStatus>);

/// Writes the destination, its distance and the state of the route: when one of them changes,
/// and twice a second for the distance.
fn status(
    time: Res<Time>,
    scene: Scene,
    (destination, route): Route,
    mut since: Local<f32>,
    mut lines: Query<&mut Text, With<Status>>,
) {
    *since += time.delta_secs();
    let empty = lines.iter().any(|text| text.0.is_empty());
    if !(destination.is_changed() || route.is_changed() || empty || *since >= REFRESH) {
        return;
    }
    *since = 0.0;
    let shown = scene.route(destination.0, *route);
    for mut text in &mut lines {
        text.set_if_neq(Text(shown.clone()));
    }
}

/// Registers the systems that fill the screen.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            rebuild,
            rows,
            tabs,
            status,
            super::readout::write,
            super::map::draw,
        )
            .chain()
            .after(super::Input)
            .run_if(ready),
    );
}
