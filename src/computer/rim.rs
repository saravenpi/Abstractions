//! Home on the map: the small house beside the home galaxy, star or planet, the room a far
//! home leaves the rest of the chart, and the house pinned to the rim of the map, with its
//! distance, in the direction of a home the map does not show.

use bevy::prelude::*;

use super::board::Board;
use super::chart::{Dot, FILL, flat};
use super::layout::{SMALLEST, measure};
use super::map::{SIZE, spot};
use super::pointer::MapView;
use crate::ui::units::distance;
use crate::ui::{Icon, Tone, icon_sized, text, theme};

/// Height in pixels of a house drawn on the map.
const HOUSE: f32 = 12.0;

/// Pixels the ship takes on either side of its point, which the house keeps clear of.
const SHIP_ROOM: f32 = 10.0;

/// Width and height in pixels of the room of the pin: its house over its distance.
const PIN: Vec2 = Vec2::new(72.0, 30.0);

/// How many times farther than a row the next row out must be for the chart to be drawn to
/// the nearer ones and leave home off the map, on the rim.
pub const APART: f32 = 1.5;

/// How far off a chart that is not to scale a home it does not list is placed, in charts.
const OFF: f32 = 1000.0;

/// Marks the pin of the rim.
#[derive(Component)]
pub(super) struct Pin;

/// Marks the text of the pin that gives the distance home.
#[derive(Component)]
pub(super) struct PinFar;

/// Returns where on a map `side` pixels wide the pin of a place drawn at `pixel` sits: on the
/// line from the middle of the map to the place, `inset` pixels inside the edge it leaves
/// through. Nothing for a place the map shows.
pub fn rim(pixel: Vec2, side: f32, inset: Vec2) -> Option<Vec2> {
    let half = Vec2::splat(0.5 * side);
    let off = pixel - half;
    if off.abs().max_element() <= half.x {
        return None;
    }
    let room = (half - inset).max(Vec2::ZERO);
    Some(half + off * (room / off.abs()).min_element())
}

/// Draws a chart of far things to its near rows when home lies past a gap: when some row
/// nearer than the row `home` is [`APART`] times nearer than the next one out, every dot
/// moves out by the same share until that row is at the edge. Home and what lies around it
/// leave the map, and `span`, the metres of one chart unit, shrinks to match.
pub fn spread(dots: &mut [(Dot, Vec2)], span: &mut Option<f64>, home: Option<usize>) {
    let own = dots
        .iter()
        .find(|(dot, _)| Some(*dot) == home.map(Dot::Row));
    let Some(own) = own.map(|(_, at)| at.length()) else {
        return;
    };
    let rows = dots.iter().filter(|(dot, _)| matches!(dot, Dot::Row(_)));
    let mut gaps: Vec<f32> = rows.map(|(_, at)| at.length()).collect();
    gaps.retain(|gap| *gap > 0.0 && *gap <= own);
    gaps.sort_by(f32::total_cmp);
    let split = gaps.windows(2).rfind(|pair| pair[1] >= pair[0] * APART);
    let Some(grow) = split.map(|pair| FILL / pair[0]) else {
        return;
    };
    for (_, at) in dots.iter_mut() {
        *at *= grow;
    }
    *span = span.map(|span| span / f64::from(grow));
}

/// Returns the home of the open tab: the home planet on the system chart, the home star on
/// the galaxy chart and the home galaxy on the universe chart, with its row when it has one.
pub fn home_row(board: &Board) -> Option<usize> {
    let home = board.home[board.tab as usize]?;
    board.lines.iter().position(|line| line.target == home)
}

/// Returns the chart point of the next place on the way home: its mark when the chart lists
/// it, and otherwise its direction from the ship, to scale or far off the chart.
pub(super) fn point(board: &Board, view: &MapView) -> Option<Vec2> {
    let (target, to) = board.beacon?;
    let row = board.lines.iter().position(|line| line.target == target);
    let dot = row.and_then(|row| view.dots.iter().find(|(dot, _)| *dot == Dot::Row(row)));
    if let Some((_, at)) = dot {
        return Some(*at);
    }
    let away = to - board.ship;
    Some(match view.span {
        Some(span) => flat(away / span),
        None => flat(away.with_y(0.0).normalize_or_zero() * f64::from(OFF)),
    })
}

/// Returns how much a name matters with home first: the home of the open chart is always
/// named, and every other name keeps its order behind it.
pub fn rank(home: bool, base: u8) -> u8 {
    if home { 0 } else { base + 1 }
}

/// Spawns a small house to the left of the mark of the home of the open chart, moved out
/// past the ship when the ship is drawn on it, so the two never overlap.
pub(super) fn badge(sheet: &mut ChildSpawnerCommands, board: &Board, view: &MapView) {
    let row = home_row(board).map(Dot::Row);
    let find = |wanted: Option<Dot>| view.dots.iter().find(|(dot, _)| Some(*dot) == wanted);
    let Some((_, at)) = find(row) else {
        return;
    };
    let scale = [1.0, view.side / SIZE][usize::from(view.side > 0.0)];
    let apart = |ship: Vec2| view.lens.to_map(ship).distance(view.lens.to_map(*at)) * scale;
    let near = find(Some(Dot::Ship)).is_some_and(|(_, ship)| apart(*ship) < SHIP_ROOM);
    let shift = [0.0, SHIP_ROOM][usize::from(near)];
    let corner = Vec2::new(-HOUSE - theme::GAP_S - shift, -0.5 * HOUSE);
    let house = icon_sized(Icon::Home, Tone::Plain, HOUSE);
    sheet
        .spawn(spot(*at, corner, Vec2::splat(HOUSE)))
        .with_child(house);
}

/// The pin of the rim, hidden until home is off the map: a house over the distance home.
pub(super) fn pin() -> impl Bundle {
    let node = Node {
        display: Display::None,
        position_type: PositionType::Absolute,
        width: px(PIN.x),
        height: px(PIN.y),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        ..default()
    };
    let far = (PinFar, text("", theme::FONT_SMALL, theme::TEXT));
    (
        Pin,
        node,
        children![icon_sized(Icon::Home, Tone::Plain, HOUSE), far],
    )
}

/// Keeps the pin on the rim toward a home the map does not show and writes its distance, and
/// hides it while home is on the map. It follows the lens as the map is dragged and zoomed.
pub(super) fn place(
    board: Res<Board>,
    view: Res<MapView>,
    mut pins: Query<&mut Node, With<Pin>>,
    mut texts: Query<&mut Text, With<PinFar>>,
) {
    if !(board.is_changed() || view.is_changed()) {
        return;
    }
    let side = [measure(SMALLEST, (0, 0)).map, view.side][usize::from(view.side > 0.0)];
    let drawn = |at: Vec2| view.lens.to_map(at) / SIZE * side;
    let at = point(&board, &view).and_then(|at| rim(drawn(at), side, 0.5 * PIN));
    let corner = at.map(|at| at - 0.5 * PIN);
    let wanted = (
        corner.map_or(Display::None, |_| Display::Flex),
        corner.map_or(auto(), |corner| px(corner.x)),
        corner.map_or(auto(), |corner| px(corner.y)),
    );
    for mut node in &mut pins {
        if (node.display, node.left, node.top) != wanted {
            (node.display, node.left, node.top) = wanted;
        }
    }
    let gap = board.beacon.map_or(0.0, |(_, to)| to.distance(board.ship));
    for mut text in &mut texts {
        text.set_if_neq(Text(distance(gap)));
    }
}
