//! The levels of the map: a planet spread flat, or the charts of a system, a galaxy and the
//! universe. The wheel zooms inside one level; a double click goes into a place and Back
//! comes out of it.

use bevy::math::{DVec3, Vec2};
use bevy::prelude::*;

use super::geo::to_chart;
use super::me::Me;
use super::raster::grid::COLS;
use crate::computer::{Board, Lens, Tab};
use crate::universe::BodyId;

/// Most screen pixels one cell of the planet picture takes at the closest zoom.
const CELL_PIXELS: f32 = 6.0;

/// Side in pixels of the square map in a 1280 by 720 window, which the closest zoom is set for.
pub(super) const REFERENCE_MAP: f32 = 720.0;

/// Closest zoom of the flat map: one cell of the picture is then [`CELL_PIXELS`] wide on the
/// map of a 1280 by 720 window, so the picture stays readable.
pub(super) const CLOSEST: f32 = CELL_PIXELS * COLS as f32 / REFERENCE_MAP;

/// Zoom of the flat map the screen opens at on the ground: the land around the player, as close
/// as the map goes.
pub const GROUND: f32 = CLOSEST;

/// Zoom of the flat map at which the planet fills the height of the map.
pub const WHOLE: f32 = 2.0;

/// Which level the map shows.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Level {
    /// The chart of the open tab of the board computer: a system, a galaxy or the universe.
    #[default]
    Chart,
    /// The flat map of this planet or moon.
    Planet(BodyId),
}

/// Run condition: the map shows a chart, or there is no map.
pub fn charting(level: Option<Res<Level>>) -> bool {
    level.is_none_or(|level| *level == Level::Chart)
}

/// Run condition: the map shows a planet.
pub fn on_planet(level: Option<Res<Level>>) -> bool {
    !charting(level)
}

/// Opens the map on the planet under the player, close on the ground, or on the system chart
/// in space.
pub(super) fn open(me: Me, mut level: ResMut<Level>, mut board: ResMut<Board>) {
    let spot = me.spot();
    *level = spot.map_or(Level::Chart, |(body, _)| Level::Planet(body));
    if spot.is_some() && board.tab != Tab::System {
        board.show(Tab::System);
    }
}

/// Returns `lens` held to the flat map of a planet: from the whole planet to [`CLOSEST`], and
/// never past the poles, so the picture always fills the view. East and west it wraps.
pub fn framed(lens: Lens) -> Lens {
    let zoom = lens.zoom.clamp(WHOLE, CLOSEST);
    let reach = 0.5 - 1.0 / zoom;
    Lens {
        zoom,
        centre: Vec2::new(lens.centre.x, lens.centre.y.clamp(-reach, reach)),
    }
}

/// Returns where the flat map of a planet looks first: on `focus` when it was asked for, on
/// the player close to the ground when they are on it, and on the whole planet otherwise.
pub fn first(focus: Option<DVec3>, here: Option<DVec3>) -> Lens {
    let whole = Lens {
        zoom: WHOLE,
        centre: Vec2::ZERO,
    };
    let spot = focus.or(here).map(to_chart);
    let lens = spot.map_or(whole, |centre| Lens {
        zoom: GROUND,
        centre,
    });
    framed(lens)
}
