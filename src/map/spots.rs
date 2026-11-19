//! Where a marker is drawn on a chart: on the mark of its planet, its star or its galaxy when
//! the chart has one, and otherwise at its place in the universe, laid out as the chart lays
//! out its rows.

use bevy::math::{DVec3, Vec2};

use crate::computer::{Board, Dot, FILL, Line, Tab, Target, ship_position, target_position};
use crate::menus::{Marker, MarkerPlace};
use crate::space::{Frame, UniPos};
use crate::universe::{BodyId, Universe};

/// Returns `offset` seen from above: east to the right and north up, as the charts draw it.
fn flat(offset: DVec3) -> Vec2 {
    Vec2::new(offset.x as f32, -offset.z as f32)
}

/// Returns the rows of a chart a body is drawn on, nearest first: the body, its star, its galaxy.
fn hosts(body: BodyId) -> [Target; 3] {
    [
        Target::Body(body),
        Target::Star(body.star),
        Target::Galaxy(body.star.galaxy),
    ]
}

/// Returns the row the marker sits on in `lines`, when it stands on a body the chart shows.
pub fn row_of(marker: &Marker, lines: &[Line]) -> Option<usize> {
    let MarkerPlace::Body(body, _) = marker.place else {
        return None;
    };
    let rows = hosts(body).into_iter();
    rows.filter_map(|target| lines.iter().position(|line| line.target == target))
        .next()
}

/// Returns where in the universe the marker is, at `time` seconds, for a world anchored in
/// `frame`.
pub fn universe_at(
    marker: &Marker,
    (universe, frame): (&Universe, Frame),
    time: f64,
) -> Option<UniPos> {
    match marker.place {
        MarkerPlace::Body(body, _) => target_position(universe, Target::Body(body), time),
        MarkerPlace::Anchor(pos) => ship_position(universe, frame, pos, time),
    }
}

/// Returns the chart point of the place `at` on the chart of `board`, with `span` metres a
/// chart unit on a far chart. A system is drawn around its star with
/// the square root of the distance; far charts are drawn around the ship to scale.
pub fn chart_point(board: &Board, span: Option<f64>, at: UniPos) -> Option<Vec2> {
    let lines = &board.lines;
    if board.tab != Tab::System {
        return span.map(|span| flat(at - board.ship) / span as f32);
    }
    let star = lines
        .iter()
        .find(|line| matches!(line.target, Target::Star(_)));
    let origin = star.or(lines.first())?.at;
    let far = lines.iter().map(|line| flat(line.at - origin).length());
    let reach = far.fold(1.0, f32::max);
    let gap = flat(at - origin);
    Some(gap.normalize_or_zero() * (gap.length() / reach).min(1.0).sqrt() * FILL)
}

/// What a view of the universe needs to place markers: the world, its frame and the time.
pub type Sky<'a> = (&'a Universe, Frame, f64);

/// What a chart is: its rows, its dots and the metres a chart unit stands for.
pub type Chart<'a> = (&'a Board, &'a [(Dot, Vec2)], Option<f64>);

/// Returns the chart point of `marker` on a chart: on the dot of the row it sits on, or at its
/// place in the universe.
pub fn place(marker: &Marker, (board, dots, span): Chart, sky: Sky) -> Option<Vec2> {
    if let Some(row) = row_of(marker, &board.lines) {
        let dot = Dot::Row(row);
        return dots
            .iter()
            .find(|(other, _)| *other == dot)
            .map(|(_, at)| *at);
    }
    let at = universe_at(marker, (sky.0, sky.1), sky.2)?;
    chart_point(board, span, at)
}
