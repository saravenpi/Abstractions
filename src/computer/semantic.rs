//! Semantic zoom: the three charts as one map. Zooming out of a whole chart opens the chart
//! around it, and zooming far into the place of the ship opens the chart inside it.

use bevy::math::Vec2;

use super::lens::Lens;
use super::tab::Tab;

/// Zoom from which one more notch into the place of the ship opens the chart inside it. A
/// chart reached by zooming out opens at this zoom, so the way back is one notch away.
pub const DIVE: f32 = 8.0;

/// Returns the chart around the chart of `tab`, when there is one.
pub fn outer(tab: Tab) -> Option<Tab> {
    match tab {
        Tab::System => Some(Tab::Galaxy),
        Tab::Galaxy => Some(Tab::Universe),
        Tab::Universe => None,
    }
}

/// Returns the chart inside the place of the ship on the chart of `tab`, when there is one.
pub fn inner(tab: Tab) -> Option<Tab> {
    match tab {
        Tab::Universe => Some(Tab::Galaxy),
        Tab::Galaxy => Some(Tab::System),
        Tab::System => None,
    }
}

/// Returns the tab the wheel opens instead of zooming: the chart around this one when the
/// wheel turns out of a chart seen whole, and the chart inside when it turns in at [`DIVE`]
/// or closer with the pointer on the place of the ship, which is what `home` says.
pub fn shift(tab: Tab, zoom: f32, notches: f32, home: bool) -> Option<Tab> {
    if notches < 0.0 && zoom <= 1.0 {
        outer(tab)
    } else if notches > 0.0 && zoom >= DIVE && home {
        inner(tab)
    } else {
        None
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
