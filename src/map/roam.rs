//! Moving the lens of the flat map of a planet: panning and zooming it without the edges of a
//! chart, since the map wraps east and west, and focusing it on a place.

use bevy::math::{DVec2, Vec2};

use super::level::{CLOSEST, WHOLE, framed};
use super::sight::Sight;
use crate::computer::SIZE;

/// Zoom a focused place is shown at at least: halfway from the whole planet to the ground.
pub const FOCUS: f32 = 3.0;

/// Returns `lens` after the flat map was dragged by `pixels`. East and west it runs on around
/// the planet; north and south it stops at the poles.
pub fn slid(lens: Sight, pixels: Vec2) -> Sight {
    let moved = DVec2::new(-f64::from(pixels.x), f64::from(pixels.y)) * 2.0
        / (f64::from(SIZE) * f64::from(lens.zoom));
    framed(Sight {
        centre: lens.centre + moved,
        ..lens
    })
}

/// Returns `lens` zoomed `factor` times closer around `pixel`: the place under that pixel
/// stays under it, unless the poles or the ends of the zoom stop the map.
pub fn scaled(lens: Sight, pixel: Vec2, factor: f32) -> Sight {
    let held = lens.to_chart(pixel);
    let zoom = (lens.zoom * factor).clamp(WHOLE, CLOSEST);
    let centre = held - (held - lens.centre) * f64::from(lens.zoom) / f64::from(zoom);
    framed(Sight { zoom, centre })
}

/// Returns the lens that focuses the flat map on the chart point `at`: centred on it, at least
/// as close as [`FOCUS`], and closer when the map already was.
pub fn focused(at: DVec2, zoom: f32) -> Sight {
    framed(Sight {
        zoom: zoom.max(FOCUS),
        centre: at,
    })
}
