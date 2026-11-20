//! The pointer on the map: pointing at a mark, clicking it, dragging the chart and zooming it
//! with the wheel.
//!
//! Pointing at a mark only lights it and names it. A click selects it, a click on empty map
//! lets go of the selection, and a double click goes into the place of the mark. The course is
//! set with Enter or the buttons on the card of the selected place.

mod click;
mod wheel;

pub use click::aimed;
pub(super) use click::{coast, press};
#[cfg(test)]
pub(super) use click::{framing, tap};
pub(super) use wheel::zoom;

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::board::Board;
use super::chart::{Dot, chart, hit, span};
use super::lens::Lens;
use super::map::{Field, SIZE};
use super::moons;
use super::open::Computer;
use super::rim::{home_row, spread};
use super::route::Target;
use super::semantic::entry;
use super::tab::Tab;
use super::view::Area;

/// Pixels around a mark within which the pointer is on it.
pub const REACH: f32 = 10.0;

/// Pixels the pointer travels with the button held before a click becomes a drag.
pub const SLOP: f32 = 3.0;

/// How much closer one line of the wheel brings the map.
pub const ZOOM_STEP: f32 = 1.15;

/// Seconds between two clicks on the same thing that still count as a double click.
pub const DOUBLE_CLICK: f32 = 0.4;

/// The key that shows the whole chart again.
pub const RESET_KEY: KeyCode = KeyCode::Digit0;

/// How the map is looked at and what the pointer does on it.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct MapView {
    /// The zoom and the centre of the map as it is drawn.
    pub lens: Lens,
    /// The zoom and the centre the map eases to.
    pub goal: Lens,
    /// The tab the lens belongs to. Another tab starts from the whole chart.
    pub tab: Tab,
    /// Every dot of the chart with its chart point, kept from the last time the table changed.
    pub dots: Vec<(Dot, Vec2)>,
    /// True while the pointer is over the map.
    pub over: bool,
    /// The pixel of the map the pointer is at.
    pub at: Vec2,
    /// The mark under the pointer.
    pub hover: Option<Dot>,
    /// Where the pointer was last seen with the left button held, since it went down on the map.
    pub held: Option<Vec2>,
    /// True once the held pointer moved far enough to drag the chart.
    pub dragged: bool,
    /// What the last click was on, and when in seconds.
    pub clicked: Option<(Option<Dot>, f32)>,
    /// The row the map last brought into view.
    pub cursor: usize,
    /// Side of the map on screen in pixels, or 0 before the window sized it.
    pub side: f32,
    /// Metres one chart unit stands for, or nothing on a chart that is not to scale.
    pub span: Option<f64>,
    /// True while the player has picked the row under the cursor, by a click or a key.
    pub picked: bool,
    /// True while that pick was a click on its mark on the map, which Escape lets go of.
    pub marked: bool,
    /// A place the map was asked to go into, by a double click or the menu, which the map takes
    /// once it has eased onto it.
    pub into: Option<Target>,
}

/// Returns the selected row: the one under the cursor once the player picked it, or the first
/// result of a search.
pub fn selected(board: &Board, view: &MapView) -> Option<usize> {
    let chosen = view.picked || !board.filter.is_empty();
    (chosen && board.cursor < board.lines.len()).then_some(board.cursor)
}

/// The square map and the room around it, which say where the pointer is.
type Under<'w, 's> = (
    Query<'w, 's, &'static RelativeCursorPosition, With<Field>>,
    Query<'w, 's, &'static RelativeCursorPosition, With<Area>>,
);

/// Follows the pointer: finds the mark under it when it moves. The mark is only lit: the
/// selection stays where it is. The pointer counts as on the map anywhere in the room around
/// the square, where a zoomed chart runs on.
pub(super) fn aim((fields, areas): Under, board: Res<Board>, mut view: ResMut<MapView>) {
    let seen = fields.iter().next();
    let room = areas.iter().next().map(|area| area.cursor_over);
    let pointer =
        seen.and_then(|field| Some((room.unwrap_or(field.cursor_over), field.normalized?)));
    let (over, at) = match pointer {
        Some((over, share)) => (over, (share + 0.5) * SIZE),
        None => (false, view.at),
    };
    let zoom = view.lens.zoom;
    let hidden = view
        .hover
        .is_some_and(|dot| !moons::seen(dot, &board.lines, zoom));
    let still = at == view.at && over == view.over && !hidden;
    if (view.over, view.at) != (over, at) {
        (view.over, view.at) = (over, at);
    }
    if still || view.held.is_some() {
        return;
    }
    let shown = view
        .dots
        .iter()
        .filter(|(dot, _)| moons::seen(*dot, &board.lines, zoom));
    let hover = hit(shown, view.lens, at, REACH).filter(|_| over);
    if hover != view.hover {
        view.hover = hover;
    }
}

/// Lays the chart out again when the table changed, to its near rows when home is far
/// beyond them. Another tab opens as the way there asks, with nothing picked, and a cursor
/// that moved to a row outside the map brings that row into view.
pub(super) fn plot(board: Res<Board>, mut view: ResMut<MapView>) {
    if !board.is_changed() {
        return;
    }
    view.dots = chart(board.tab, &board.lines, board.ship);
    view.span = span(board.tab, &board.lines, board.ship);
    let (view, far) = (&mut *view, board.tab != Tab::System);
    spread(
        &mut view.dots,
        &mut view.span,
        home_row(&board).filter(|_| far),
    );
    if view.tab != board.tab {
        (view.lens, view.goal) = entry(view.tab, board.tab);
        view.tab = board.tab;
        (view.hover, view.clicked, view.picked, view.marked) = (None, None, false, false);
    }
    if view.cursor == board.cursor {
        return;
    }
    view.cursor = board.cursor;
    let row = Dot::Row(board.cursor);
    if let Some((_, at)) = view.dots.iter().find(|(dot, _)| *dot == row) {
        view.goal = view.goal.revealing(*at);
    }
}

/// Registers the lens and the systems of the pointer. An opened screen shows the whole chart.
pub(super) fn plugin(app: &mut App) {
    let whole = |mut view: ResMut<MapView>| {
        *view = MapView {
            side: view.side,
            tab: view.tab,
            ..default()
        }
    };
    app.init_resource::<MapView>()
        .add_systems(OnEnter(Computer::Open), whole)
        .add_systems(
            Update,
            (aim, press, zoom, coast)
                .chain()
                .in_set(super::Input)
                .before(super::keys::navigate)
                .run_if(super::open::ready)
                .run_if(crate::map::charting),
        );
}
