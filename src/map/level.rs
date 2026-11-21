//! The levels of the map: a planet spread flat, or the charts of a system, a galaxy and the
//! universe. The wheel zooms inside one level; a double click goes into a place and Back
//! comes out of it.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::geo::to_chart;
use super::me::Me;
use super::sight::Sight;
use crate::computer::{Board, Tab};
use crate::universe::BodyId;

/// Side in pixels of the square map in a 1280 by 720 window, which the closest zoom is set for.
pub(super) const REFERENCE_MAP: f32 = 720.0;

/// Closest zoom of the flat map: the map then spans one part in 81920 of the way around the
/// planet, about 490 m on a planet the size of Earth, a pixel of a 1280 by 720 window being
/// 0.7 m, so the base, the pad, a pin and a herd beside them are each their own mark. The
/// tiles of the planet keep it sharp, a pixel of the finest tile being then under two pixels
/// of the map. The charts of the board computer stop at their own `MAX_ZOOM`.
pub(super) const CLOSEST: f32 = 81_920.0;

/// Zoom of the flat map the screen opens at on the ground: one part in 8192 of the way around,
/// about 5 km on an Earth, so the ground a player uncovers on foot, 300 m around them, and the
/// 2 km around the home base fill it instead of hiding under their marker. The wheel goes ten
/// times closer, to [`CLOSEST`].
pub const GROUND: f32 = 8192.0;

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
pub fn framed(lens: Sight) -> Sight {
    let zoom = lens.zoom.clamp(WHOLE, CLOSEST);
    let reach = 0.5 - 1.0 / f64::from(zoom);
    Sight {
        zoom,
        centre: DVec2::new(lens.centre.x, lens.centre.y.clamp(-reach, reach)),
    }
}

/// Returns where the flat map of a planet looks first: on `focus` when it was asked for, on
/// the player close to the ground when they are on it, and on the whole planet otherwise.
pub fn first(focus: Option<DVec3>, here: Option<DVec3>) -> Sight {
    let whole = Sight {
        zoom: WHOLE,
        centre: DVec2::ZERO,
    };
    let spot = focus.or(here).map(to_chart);
    let lens = spot.map_or(whole, |centre| Sight {
        zoom: GROUND,
        centre,
    });
    framed(lens)
}
