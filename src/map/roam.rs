//! Moving the lens of the flat map of a planet: panning and zooming it without the edges of a
//! chart, since the map wraps east and west, and focusing it on a place.

use bevy::math::Vec2;

use super::level::{CLOSEST, WHOLE, framed};
use crate::computer::{Lens, SIZE};

/// Zoom a focused place is shown at at least: halfway from the whole planet to the ground.
pub const FOCUS: f32 = 3.0;

/// Returns `lens` after the flat map was dragged by `pixels`. East and west it runs on around
/// the planet; north and south it stops at the poles.
pub fn slid(lens: Lens, pixels: Vec2) -> Lens {
    let moved = Vec2::new(-pixels.x, pixels.y) * 2.0 / (SIZE * lens.zoom);
    framed(Lens {
        centre: lens.centre + moved,
        ..lens
    })
}

/// Returns `lens` zoomed `factor` times closer around `pixel`: the place under that pixel
/// stays under it, unless the poles or the ends of the zoom stop the map.
pub fn scaled(lens: Lens, pixel: Vec2, factor: f32) -> Lens {
    let held = lens.to_chart(pixel);
    let zoom = (lens.zoom * factor).clamp(WHOLE, CLOSEST);
    let centre = held - (held - lens.centre) * lens.zoom / zoom;
    framed(Lens { zoom, centre })
}

/// Returns the lens that focuses the flat map on the chart point `at`: centred on it, at least
/// as close as [`FOCUS`], and closer when the map already was.
pub fn focused(at: Vec2, zoom: f32) -> Lens {
    framed(Lens {
        zoom: zoom.max(FOCUS),
        centre: at,
    })
}
