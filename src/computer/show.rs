//! Filling the computer screen: building the rows and writing every text when it changes.

use bevy::prelude::*;

use super::board::Board;
use super::course::progress;
use super::format::{clip, distance};
use super::home::{arranged, beacon, homes, pinned};
use super::moons::is_moon;
use super::open::ready;
use super::pointer::{MapView, selected};
use super::results::TITLE_CHARS;
use super::route::{Destination, HOME_LABEL, RouteStatus, Target};
use super::rows::Line;
use super::scene::Scene;
use super::sift;
use super::tab::Tab;
use super::view::{Action, Cell, Part, Slot, Status, Summary};
use crate::ui::{Icon, Tone};

/// Seconds between two refreshes of what changes while flying: the distances of the system tab
/// and of the route line.
const REFRESH: f32 = 0.5;

/// Builds the rows when the tab asks for them, when the chart of known systems is finished,
/// when the ship enters or leaves a system or a galaxy, and twice a second on the system tab.
/// The row of home is pinned first while the ship is away from it.
fn rebuild(time: Res<Time>, scene: Scene, mut board: ResMut<Board>, mut since: Local<f32>) {
    *since += time.delta_secs();
    let moving = board.tab == Tab::System && *since >= REFRESH;
    let here = scene.here();
    if !(board.stale || moving || scene.charted() || here != board.here) {
        return;
    }
    *since = 0.0;
    let (heading, lines) = scene.table(board.tab, board.inside);
    let pin = pinned(board.tab, here, &scene.universe);
    let lines = arranged(lines, pin, &board.filter, board.sort);
    board.ship = scene.ship();
    board.here = here;
    board.home = homes(&scene.universe);
    board.beacon = beacon(&scene.universe, here, scene.clock.seconds);
    board.worlds = scene.worlds(&lines);
    board.fill(heading, lines);
}

/// What the name of a moon starts with, so that it reads as part of its planet.
const INDENT: &str = "  ";

/// Returns the icon of a row: a pin on the destination, the ship where the ship is, and
/// otherwise what the row is, a star, a planet or a galaxy.
fn sign(target: Target, chosen: bool, here: bool) -> Icon {
    match (chosen, here, target) {
        (true, _, _) => Icon::MapPin,
        (_, true, _) => Icon::Send,
        (_, _, Target::Star(_)) => Icon::Star,
        (_, _, Target::Body(body)) if body.is_moon() => Icon::Moon,
        (_, _, Target::Body(_)) => Icon::Planet,
        (_, _, Target::Galaxy(_)) => Icon::Map,
    }
}

/// Returns the text and the tone of one cell of `line`. The destination and the place of the
/// ship are marked, a row that says home in its name wears the house unless it is the
/// destination, a place never visited is dimmed, the name of a moon is indented under its
/// planet, and a long name is cut short to fit its column.
pub(super) fn cell_text(line: &Line, part: Part, (chosen, here): (bool, bool)) -> (String, Tone) {
    let seen = match (chosen, line.visited || here) {
        (true, _) => Tone::Accent,
        (false, true) => Tone::Plain,
        (false, false) => Tone::Muted,
    };
    match part {
        Part::Mark if !chosen && line.title.ends_with(HOME_LABEL) => (Icon::Home.glyph(), seen),
        Part::Mark => (sign(line.target, chosen, here).glyph(), seen),
        Part::Title if is_moon(line.target) => {
            (clip(&format!("{INDENT}{}", line.title), TITLE_CHARS), seen)
        }
        Part::Title => (clip(&line.title, TITLE_CHARS), seen),
        Part::Distance => (distance(line.distance), Tone::Plain),
    }
}

/// What the rows are drawn from.
type Source<'w> = (Res<'w, Board>, Res<'w, Destination>);

/// Writes the texts of the drawn rows and hides the rows past the end of the table. Does
/// nothing while neither the table nor the destination changed.
fn rows(
    (board, destination): Source,
    fresh: Query<(), Added<Slot>>,
    mut cells: Query<(&Cell, &mut Text, &mut Tone)>,
    mut slots: Query<(&Slot, &mut Node)>,
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
    for (slot, mut node) in &mut slots {
        let display = match board.at(slot.0) {
            Some(_) if slot.0 < board.window() => Display::Flex,
            _ => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
    }
}

/// Returns the line under the tabs: which rows are drawn, their order and the filter.
fn summary(board: &Board) -> String {
    let (first, count) = (board.top + 1, board.lines.len());
    let last = (board.top + board.window()).min(count);
    let view = (board.filter.as_str(), board.sort);
    sift::summary(&board.heading, (first, last, count), view)
}

/// Writes the summary line over the results, when the table changed.
fn tabs(
    board: Res<Board>,
    fresh: Query<(), Added<Summary>>,
    mut lines: Query<&mut Text, With<Summary>>,
) {
    if !(board.is_changed() || !fresh.is_empty()) {
        return;
    }
    for mut text in &mut lines {
        text.set_if_neq(Text(summary(&board)));
    }
}

/// What the route line is drawn from.
type Route<'w> = (Res<'w, Destination>, Res<'w, RouteStatus>);

/// The lines the course and the progress of the route are written on.
type Written = Or<(With<Status>, With<Action>)>;

/// Writes the course with its distance and the state of the route, and under the map the
/// progress of the route while it is flown: when the course, the route or the selection
/// changes, and twice a second for the distance. Without a course the course line stays empty
/// while a row is selected, since the card of the row already offers to set the course.
fn status(
    (time, board, view): (Res<Time>, Res<Board>, Res<MapView>),
    scene: Scene,
    (destination, route): Route,
    mut since: Local<f32>,
    mut lines: Query<(&mut Text, Has<Action>), Written>,
) {
    *since += time.delta_secs();
    let changed = destination.is_changed() || route.is_changed() || board.is_changed();
    if !(changed || *since >= REFRESH) {
        return;
    }
    *since = 0.0;
    let offered = destination.0.is_none() && selected(&board, &view).is_some();
    let shown = match offered {
        true => String::new(),
        false => scene.route(destination.0, *route),
    };
    let name = destination.0.map(|target| target.name(&scene.universe));
    let going = name.map_or(String::new(), |name| progress(&name, *route));
    for (mut text, action) in &mut lines {
        let wanted = if action { &going } else { &shown };
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
            (super::pointer::plot, super::glide::ease).chain(),
            super::draw::draw,
            (super::glide::slide, super::glide::rule, super::rim::place),
            (super::arrow::turn, super::hover::light),
            super::chrome::fit,
        )
            .chain()
            .after(super::Input)
            .run_if(ready),
    );
}
