//! The small drawings of the pack: a tool's silhouette from its parts, and a thin wear bar.

use bevy::prelude::*;

use crate::craft::build::Form;
use crate::ui::theme;

/// Width in pixels of the room a silhouette is drawn in.
pub const WIDE: f32 = 32.0;

/// Height in pixels of the room a silhouette is drawn in.
pub const HIGH: f32 = 16.0;

/// Pixels per metre a silhouette is drawn at.
const SCALE: f32 = 18.0;

/// Height in pixels of a shaft and of a wear bar.
const THIN: f32 = 2.0;

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

/// Returns the blocks of a tool of `shape`, shaft length, head size and form in metres, or of a
/// heap of material when there is no shape.
pub fn pieces(shape: Option<(f32, f32, Form)>) -> Vec<(Node, BackgroundColor)> {
    let Some((length, size, form)) = shape else {
        return vec![block(HIGH / 2.0, HIGH / 2.0, theme::MUTED)];
    };
    let head = (size * SCALE).clamp(3.0, HIGH);
    let (wide, high) = match form {
        Form::Edge => (head * 0.6, head),
        Form::Point => (head, (head * 0.3).max(THIN + 1.0)),
        Form::Mass => (head, head),
    };
    let shaft = (length * SCALE).clamp(4.0, WIDE - wide);
    vec![
        block(shaft, THIN, theme::TEXT),
        block(wide, high, theme::TEXT),
    ]
}

/// Returns the silhouette of `shape` in its fixed room.
pub fn silhouette(shape: Option<(f32, f32, Form)>) -> impl Bundle {
    let node = Node {
        width: px(WIDE),
        height: px(HIGH),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        ..default()
    };
    (node, Children::spawn(SpawnIter(pieces(shape).into_iter())))
}

/// Returns a wear bar `width` pixels wide, filled with what is left of a tool `worn` 0 to 1.
pub fn wear_bar(width: f32, worn: f32) -> impl Bundle {
    let left = (1.0 - worn).clamp(0.0, 1.0) * width;
    let (node, color) = block(width, THIN, theme::FAINT);
    (node, color, children![block(left, THIN, theme::TEXT)])
}
