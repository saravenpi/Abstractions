//! Filling the computer screen: building the rows and writing every text when it changes.

use bevy::prelude::*;

use super::board::Board;
use super::format::distance;
use super::open::ready;
use super::route::{Destination, RouteStatus, Target};
use super::rows::Line;
use super::scene::Scene;
use super::sift::{self, arrange};
use super::tab::Tab;
use super::view::{Cell, Crumbs, Part, Slot, Status, Summary, TabLink};
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
    let lines = arrange(lines, &board.filter, board.sort);
    board.ship = scene.ship();
    board.here = scene.here();
    board.worlds = scene.worlds(&lines);
    board.fill(heading, lines);
}

/// Returns the icon of a row: a pin on the destination, the ship where the ship is, and
/// otherwise what the row is, a star, a planet or a galaxy.
fn sign(target: Target, chosen: bool, here: bool) -> Icon {
    match (chosen, here, target) {
        (true, _, _) => Icon::MapPin,
        (_, true, _) => Icon::Send,
        (_, _, Target::Star(_)) => Icon::Star,
        (_, _, Target::Body(_)) => Icon::Earth,
        (_, _, Target::Galaxy(_)) => Icon::Map,
    }
}

/// Returns the text and the tone of one cell of `line`. The destination and the place of the
/// ship are marked, and a place never visited is dimmed.
pub(super) fn cell_text(line: &Line, part: Part, (chosen, here): (bool, bool)) -> (String, Tone) {
    let seen = match (chosen, line.visited || here) {
        (true, _) => Tone::Accent,
        (false, true) => Tone::Plain,
        (false, false) => Tone::Muted,
    };
    match part {
        Part::Mark => (sign(line.target, chosen, here).glyph(), seen),
        Part::Title => (line.title.clone(), seen),
        Part::Distance => (distance(line.distance), Tone::Plain),
        Part::Detail => (line.facts().to_string(), Tone::Muted),
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
        let marks = (
            destination.0 == Some(line.target),
            board.here.contains(&Some(line.target)),
        );
        let (shown, wanted) = cell_text(line, *part, marks);
        text.set_if_neq(Text(shown));
        tone.set_if_neq(wanted);
    }
    for (slot, mut node, mut selected) in &mut slots {
        let display = match board.at(slot.0) {
            Some(_) if slot.0 < board.window() => Display::Flex,
            _ => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
        selected.set_if_neq(Selected(board.top + slot.0 == board.cursor));
    }
}

/// Returns the line under the tabs: which rows are drawn, their order and the filter.
fn summary(board: &Board) -> String {
    let (first, count) = (board.top + 1, board.lines.len());
    let last = (board.top + board.window()).min(count);
    let view = (board.filter.as_str(), board.sort);
    sift::summary(&board.heading, (first, last, count), view)
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

/// The lines the course and the place of the ship are written on.
type Written = Or<(With<Status>, With<Crumbs>)>;

/// Writes the course with its distance and the state of the route, and where the ship is:
/// when the course or the route changes, and twice a second for the distance.
fn status(
    time: Res<Time>,
    scene: Scene,
    (destination, route): Route,
    mut since: Local<f32>,
    mut lines: Query<(&mut Text, Has<Crumbs>), Written>,
) {
    *since += time.delta_secs();
    let empty = lines.iter().any(|(text, _)| text.0.is_empty());
    if !(destination.is_changed() || route.is_changed() || empty || *since >= REFRESH) {
        return;
    }
    *since = 0.0;
    let (shown, place) = (scene.route(destination.0, *route), scene.place());
    for (mut text, crumbs) in &mut lines {
        let wanted = if crumbs { &place } else { &shown };
        text.set_if_neq(Text(wanted.clone()));
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
            super::pointer::plot,
            super::draw::draw,
            (super::pointer::slide, super::pointer::rule),
            (super::chrome::fit, super::chrome::offer),
        )
            .chain()
            .after(super::Input)
            .run_if(ready),
    );
}
