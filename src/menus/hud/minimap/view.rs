//! From the ground to the map: where a point around the player sits on a map that turns with
//! the heading, how a far mark pins to the rim, and how the ground image is sized, moved and
//! turned to match.
//!
//! Ground offsets are metres east and north of the player. Map points run from -1 to 1 on both
//! axes with the heading up, like the chart of the board computer, so its `spot` places them.

use bevy::math::Vec2;

/// Share of the half width of the map kept between a pinned mark and the frame.
pub const INSET: f32 = 0.14;

/// How the image of the ground sits in the frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    /// Width and height of the image as a percentage of the frame.
    pub side: f32,
    /// How far the image is moved right and down, as a percentage of its own size.
    pub shift: Vec2,
    /// Clockwise turn of the image in radians.
    pub turn: f32,
}

/// Returns the heading in radians, clockwise from north, of a direction given as metres east
/// and north. Nothing for a direction with no length.
pub fn heading(ahead: Vec2) -> Option<f32> {
    (ahead.length_squared() > 1e-12).then(|| ahead.x.atan2(ahead.y))
}

/// Returns the map point of a ground point `offset` metres east and north of the player, on
/// a map that shows `radius` metres to each side with `heading` up.
pub fn to_view(offset: Vec2, heading: f32, radius: f32) -> Vec2 {
    let (sin, cos) = heading.sin_cos();
    let right = offset.x * cos - offset.y * sin;
    let up = offset.x * sin + offset.y * cos;
    Vec2::new(right, up) / radius
}

/// Returns where a mark at the map point `at` is drawn, and whether it was pinned: a mark
/// past the rim waits on the rim in its direction.
pub fn pin(at: Vec2) -> (Vec2, bool) {
    let (reach, edge) = (at.abs().max_element(), 1.0 - INSET);
    match reach > edge {
        true => (at * edge / reach, true),
        false => (at, false),
    }
}

/// Returns the point of the rim that lies in the direction `toward`, given as metres east and
/// north, on a map with `heading` up. The north mark sits there.
pub fn rim(toward: Vec2, heading: f32) -> Vec2 {
    let at = to_view(toward, heading, 1.0);
    let reach = at.abs().max_element().max(f32::EPSILON);
    at * (1.0 - INSET) / reach
}

/// Returns how to draw an image of the ground that reaches `half` metres to each side of its
/// middle, for a player `at` metres east and north of that middle, on a map that shows
/// `radius` metres to each side with `heading` up.
pub fn fit(half: f32, at: Vec2, heading: f32, radius: f32) -> Fit {
    let middle = to_view(-at, heading, 2.0 * half);
    Fit {
        side: half / radius * 100.0,
        shift: Vec2::new(middle.x, -middle.y) * 100.0,
        turn: -heading,
    }
}
