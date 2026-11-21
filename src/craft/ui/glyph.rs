//! The drawings of a tool: its silhouette from its parts, a shaft and a head drawn to scale in a
//! fixed room, small in a hotbar slot and large on a tool card.

use bevy::prelude::*;

use super::scene::Shape;
use crate::craft::build::Form;
use crate::ui::theme;

/// A room a silhouette is drawn in: its width and height in pixels, and pixels per metre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Room {
    /// Width in pixels.
    pub wide: f32,
    /// Height in pixels.
    pub high: f32,
    /// Pixels per metre.
    pub scale: f32,
}

/// The room of a hotbar slot.
pub const SMALL: Room = Room {
    wide: 32.0,
    high: 16.0,
    scale: 18.0,
};

/// The room of a tool card.
pub const LARGE: Room = Room {
    wide: 96.0,
    high: 32.0,
    scale: 56.0,
};

/// Returns one filled block of `width` by `height` pixels in `color`.
fn block(width: f32, height: f32, color: Color) -> (Node, BackgroundColor) {
    let node = Node {
        width: px(width),
        height: px(height),
        flex_shrink: 0.0,
        ..default()
    };
    (node, BackgroundColor(color))
}

/// Returns the blocks of a tool of `shape` in `room`: a shaft an eighth of the room high, then
/// its head, an upright blade, a flat point or a square weight.
pub fn pieces(shape: Shape, room: Room) -> Vec<(Node, BackgroundColor)> {
    let Some((length, size, form)) = shape else {
        return Vec::new();
    };
    let thin = (room.high / 8.0).max(2.0);
    let head = (size * room.scale).clamp(thin * 1.5, room.high);
    let (wide, high) = match form {
        Form::Edge => (head * 0.6, head),
        Form::Point => (head, (head * 0.3).max(thin + 1.0)),
        Form::Mass => (head, head),
    };
    let shaft = (length * room.scale).clamp(thin * 2.0, room.wide - wide);
    vec![
        block(shaft, thin, theme::TEXT),
        block(wide, high, theme::TEXT),
    ]
}

/// Returns the silhouette of `shape` in `room`, empty when there is no tool.
pub fn silhouette(shape: Shape, room: Room) -> impl Bundle {
    let node = Node {
        width: px(room.wide),
        height: px(room.high),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        ..default()
    };
    (
        node,
        Children::spawn(SpawnIter(pieces(shape, room).into_iter())),
    )
}
