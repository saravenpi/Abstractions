//! Finding what is under the pointer at a right click: a marker or bare ground on a planet,
//! a mark or empty space on a chart.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Under;
use crate::computer::{Board, Dot, MapView};
use crate::map::geo::{from_chart, wrapped};
use crate::map::level::Level;
use crate::map::pins::Pins;
use crate::map::surface::Surface;
use crate::menus::{MarkerKind, Markers};
use crate::universe::BodyId;

/// Returns what is under the pointer on the chart: the mark of a row, or empty chart.
pub fn on_chart(view: &MapView, board: &Board) -> Under {
    let row = match view.hover {
        Some(Dot::Row(index)) => board.lines.get(index),
        _ => None,
    };
    row.map_or(Under::Chart(view.lens.to_chart(view.at)), |line| {
        Under::Place(line.target)
    })
}

/// Returns how many markers of `kind` come before the marker at `index`, which is its place
/// in the list it was made from.
fn rank(markers: &Markers, index: usize, kind: MarkerKind) -> usize {
    let before = markers.0.iter().take(index);
    before.filter(|marker| marker.kind == kind).count()
}

/// Returns what is under the pointer on the flat map of `body`: the marker of `pins` it is on,
/// or the ground under it, or nothing off the planet. A marker that is neither the waypoint of
/// this player, a favourite nor a player counts as the ground it stands on.
pub fn on_planet(body: BodyId, surface: &Surface, pins: &Pins, markers: &Markers) -> Option<Under> {
    let Some(pin) = surface.hover.and_then(|index| pins.0.get(index)) else {
        return surface.point.map(|dir| Under::Ground(body, dir));
    };
    let dir = from_chart(wrapped(pin.at))?;
    let marker = pin
        .marker
        .and_then(|index| Some((index, markers.0.get(index)?)));
    let under = match marker {
        Some((_, marker)) if marker.kind == MarkerKind::Waypoint && marker.owner == 0 => {
            Under::Mine(body, dir)
        }
        Some((index, marker)) if marker.kind == MarkerKind::Favourite => {
            Under::Favourite(rank(markers, index, MarkerKind::Favourite))
        }
        Some((index, marker)) if marker.kind == MarkerKind::Friend => {
            Under::Player(rank(markers, index, MarkerKind::Friend), body)
        }
        _ => Under::Ground(body, dir),
    };
    Some(under)
}

/// What a right click finds under the pointer.
#[derive(SystemParam)]
pub struct Pointed<'w> {
    /// The level shown.
    level: Res<'w, Level>,
    /// The chart and the pointer on it.
    view: Res<'w, MapView>,
    /// The rows of the chart.
    board: Res<'w, Board>,
    /// The flat map and the pointer on it.
    surface: Res<'w, Surface>,
    /// The markers drawn on the flat map.
    pins: Res<'w, Pins>,
    /// Every marker.
    markers: Res<'w, Markers>,
}

impl Pointed<'_> {
    /// Returns what is under the pointer on the level shown, while it is over the map.
    pub fn under(&self) -> Option<Under> {
        match *self.level {
            Level::Chart => self.view.over.then(|| on_chart(&self.view, &self.board)),
            Level::Planet(body) => on_planet(body, &self.surface, &self.pins, &self.markers)
                .filter(|_| self.surface.over),
        }
    }
}
