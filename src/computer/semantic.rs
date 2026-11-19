//! The charts as levels of one map: the chart around each one, the zoom that frames a place
//! with what is inside it, and how a chart opens when the tabs move to the chart around it.

use bevy::math::Vec2;

use super::lens::Lens;
use super::tab::Tab;

/// Zoom that frames a place on a chart with what lies around it. A chart moved out to opens at
/// this zoom, close on the ship.
pub const DIVE: f32 = 8.0;

/// Returns the chart around the chart of `tab`, when there is one.
pub fn outer(tab: Tab) -> Option<Tab> {
    match tab {
        Tab::System => Some(Tab::Galaxy),
        Tab::Galaxy => Some(Tab::Universe),
        Tab::Universe => None,
    }
}

/// Returns the lens the chart of `to` opens with and the lens it eases to, coming from the
/// chart of `from`. Coming from the chart inside, it opens close on the ship, in the middle,
/// and pulls back; any other way it opens whole.
pub fn entry(from: Tab, to: Tab) -> (Lens, Lens) {
    if outer(from) != Some(to) {
        return (Lens::default(), Lens::default());
    }
    let close = |zoom: f32| Lens {
        zoom,
        centre: Vec2::ZERO,
    };
    (close(DIVE), close(DIVE * 0.5))
}
