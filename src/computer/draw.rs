//! Drawing the chart on the sheet of the map: the orbits, the course, the marks and the names.
//!
//! The hierarchy is black and white: orbits are hairlines, a star is larger than a planet and
//! a planet larger than a moon, the row under the cursor or the pointer wears a frame, the
//! destination a pin, and the course is a dotted line from the ship with the distance on it.
//! A name that would cover a more important one is not drawn, nor a moon too close to its
//! planet to tell apart.

use bevy::prelude::*;

use super::board::Board;
use super::chart::Dot;
use super::format::distance;
use super::labels::{keep, priority, room};
use super::lens::Lens;
use super::map::{SIZE, Sheet, body, frame, name, ring, ship, spot};
use super::moons::{is_moon, seen};
use super::orbits::satellites;
use super::pointer::MapView;
use super::route::{Destination, Target};
use super::tab::Tab;
use crate::ui::{Icon, Tone, icon, theme};

/// Dots of the line that shows the course.
const DASHES: usize = 24;

/// Pixels between a mark and its name.
const GAP: f32 = 10.0;

/// Side in pixels of the frame around the row under the cursor or the pointer.
const FRAME: f32 = 18.0;

/// Returns the side in pixels of the mark of a star, a planet or a galaxy, larger while it is
/// under the cursor or the pointer. A star is drawn larger than the planets around it, and a
/// planet larger than its moons.
pub fn side(target: Target, under: bool) -> f32 {
    match (target, is_moon(target), under) {
        (Target::Star(_), _, false) => 8.0,
        (Target::Star(_), _, true) => 10.0,
        (_, true, false) => 3.0,
        (_, true, true) => 6.0,
        (_, false, false) => 4.0,
        (_, false, true) => 8.0,
    }
}

/// What one drawing of the chart reads.
struct Picture<'a> {
    /// The rows and the cursor.
    board: &'a Board,
    /// The dots, the lens and the pointer.
    view: &'a MapView,
    /// The destination.
    chosen: Option<Target>,
}

impl Picture<'_> {
    /// Returns true when the row `index` is under the cursor or under the pointer.
    fn under(&self, index: usize) -> bool {
        index == self.board.cursor || self.view.hover == Some(Dot::Row(index))
    }

    /// Returns the chart point of `dot`, when the chart has it.
    fn place(&self, dot: Dot) -> Option<Vec2> {
        let found = self.view.dots.iter().find(|(other, _)| *other == dot);
        found.map(|(_, at)| *at)
    }
}

/// Spawns the orbit of every planet of a system chart as a hairline around the star, then
/// what each planet carries: its rings and its moons.
fn rings(sheet: &mut ChildSpawnerCommands, picture: &Picture) {
    let (board, view) = (picture.board, picture.view);
    if board.tab != Tab::System {
        return;
    }
    let planet = |index: usize| {
        let target = board.lines.get(index).map(|line| line.target);
        target.is_some_and(|target| matches!(target, Target::Body(_)) && !is_moon(target))
    };
    for (dot, at) in &view.dots {
        if matches!(dot, Dot::Row(index) if planet(*index)) {
            sheet.spawn(ring(at.length()));
        }
    }
    let chart = (
        view.dots.as_slice(),
        board.lines.as_slice(),
        board.worlds.as_slice(),
    );
    satellites(sheet, chart, view.lens.zoom);
}

/// Spawns the pin on the destination, and the dotted line from the ship to it with the
/// distance at its middle. A destination that is not on this chart draws nothing.
fn course(sheet: &mut ChildSpawnerCommands, picture: &Picture) {
    let lines = &picture.board.lines;
    let row = lines
        .iter()
        .position(|line| Some(line.target) == picture.chosen);
    let Some((index, to)) = row.and_then(|row| Some((row, picture.place(Dot::Row(row))?))) else {
        return;
    };
    let pin = spot(
        to,
        Vec2::new(-0.5, -1.25) * theme::ICON,
        Vec2::splat(theme::ICON),
    );
    sheet.spawn(pin).with_child(icon(Icon::MapPin, Tone::Plain));
    let Some(from) = picture.place(Dot::Ship) else {
        return;
    };
    for step in 1..DASHES {
        let at = from.lerp(to, step as f32 / DASHES as f32);
        let node = spot(at, Vec2::splat(-1.0), Vec2::splat(2.0));
        sheet.spawn((node, BackgroundColor(theme::MUTED)));
    }
    name(
        sheet,
        from.lerp(to, 0.5),
        GAP,
        &distance(lines[index].distance),
    );
}

/// Spawns the ship and the mark of every row. The row under the cursor or the pointer is
/// larger and framed, and a place never visited is dimmed.
fn marks(sheet: &mut ChildSpawnerCommands, picture: &Picture) {
    let (board, zoom) = (picture.board, picture.view.lens.zoom);
    let shown = picture.view.dots.iter();
    for (dot, at) in shown.filter(|(dot, _)| seen(*dot, &board.lines, zoom)) {
        let row = match *dot {
            Dot::Row(index) => board.lines.get(index).map(|line| (index, line)),
            Dot::Ship => {
                ship(sheet, *at, picture.view.hover == Some(Dot::Ship));
                None
            }
            Dot::Orbit => None,
        };
        let Some((index, line)) = row else {
            continue;
        };
        let under = picture.under(index);
        let color = match line.visited || under {
            true => theme::TEXT,
            false => theme::MUTED,
        };
        let world = board.worlds.get(index).and_then(Option::as_ref);
        body(sheet, *at, (side(line.target, under), color), world);
        if under {
            frame(sheet, *at, FRAME);
        }
    }
}

/// Spawns the names that fit: every row in view asks for one, the most important first, and
/// a name that would cover one already drawn is left out.
fn labels(sheet: &mut ChildSpawnerCommands, picture: &Picture) {
    let (view, board) = (picture.view, picture.board);
    let scale = if view.side > 0.0 {
        view.side / SIZE
    } else {
        1.0
    };
    let pixel = |at: Vec2| view.lens.to_map(at);
    let inside = |at: Vec2| pixel(at).min_element() >= 0.0 && pixel(at).max_element() <= SIZE;
    let drawn = |dot: Dot, at: Vec2| inside(at) && seen(dot, &board.lines, view.lens.zoom);
    let mut wanted: Vec<(u8, Vec2, &str)> = Vec::new();
    for (dot, at) in view.dots.iter().filter(|(dot, at)| drawn(*dot, *at)) {
        let Dot::Row(index) = *dot else {
            continue;
        };
        if let Some(line) = board.lines.get(index) {
            let chosen = Some(line.target) == picture.chosen;
            let rank = priority(picture.under(index), chosen, line.visited);
            wanted.push((rank, *at, line.title.as_str()));
        }
    }
    wanted.sort_by_key(|label| label.0);
    let rooms = wanted
        .iter()
        .map(|(_, at, title)| room(pixel(*at) * scale, GAP, title.len()));
    let rooms: Vec<Rect> = rooms.collect();
    for ((_, at, title), shown) in wanted.iter().zip(keep(&rooms)) {
        if shown {
            name(sheet, *at, GAP, title);
        }
    }
}

/// Draws the chart of the open tab again whenever the table, the cursor, the destination or
/// the mark under the pointer changed, when the lens comes to rest somewhere new, and when
/// the panel was just opened.
pub(super) fn draw(
    mut commands: Commands,
    (board, chosen, view): (Res<Board>, Res<Destination>, Res<MapView>),
    fresh: Query<(), Added<Sheet>>,
    sheets: Query<Entity, With<Sheet>>,
    mut last: Local<(Option<Dot>, Lens)>,
) {
    let rest = view.held.is_none() && view.lens == view.goal;
    let lens = if rest { view.lens } else { last.1 };
    let now = (view.hover, lens);
    if !(board.is_changed() || chosen.is_changed() || *last != now || !fresh.is_empty()) {
        return;
    }
    *last = now;
    let chosen = chosen.0;
    let (board, view) = (&*board, &*view);
    let picture = Picture {
        board,
        view,
        chosen,
    };
    for sheet in &sheets {
        let mut sheet = commands.entity(sheet);
        sheet.despawn_related::<Children>().with_children(|sheet| {
            rings(sheet, &picture);
            course(sheet, &picture);
            marks(sheet, &picture);
            labels(sheet, &picture);
        });
    }
}
