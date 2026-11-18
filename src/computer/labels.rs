//! Names on the map that never overlap, and the scale bar: which labels are drawn when two
//! would collide, and how long a round distance is on the map.

use bevy::math::{Rect, Vec2};

use super::layout::LINE_HEIGHT;
use crate::ui::theme;

/// Width in pixels of one character of a label.
pub const GLYPH: f32 = theme::FONT_SMALL * theme::GLYPH_ASPECT;

/// Most labels drawn at once, so a crowded galaxy stays readable and cheap to lay out.
pub const MOST_LABELS: usize = 48;

/// Longest scale bar in pixels. The bar is never shorter than two fifths of it.
pub const BAR: f64 = 80.0;

/// Returns how much a label matters, the lowest first: the row under the cursor or the
/// pointer, then the destination, then the visited places, then the others.
pub fn priority(under: bool, chosen: bool, visited: bool) -> u8 {
    match (under, chosen, visited) {
        (true, _, _) => 0,
        (_, true, _) => 1,
        (_, _, true) => 2,
        _ => 3,
    }
}

/// Returns the room a label of `chars` characters takes when its left edge is `gap` pixels to
/// the right of the pixel `at` and its middle at the height of `at`.
pub fn room(at: Vec2, gap: f32, chars: usize) -> Rect {
    let corner = at + Vec2::new(gap, -0.5 * LINE_HEIGHT);
    Rect::from_corners(
        corner,
        corner + Vec2::new(chars as f32 * GLYPH, LINE_HEIGHT),
    )
}

/// Returns, for each label in order of importance, whether it is drawn: a label that would
/// cover one already drawn is hidden, and so is every label past [`MOST_LABELS`].
pub fn keep(labels: &[Rect]) -> Vec<bool> {
    let mut drawn: Vec<Rect> = Vec::new();
    let mut kept = Vec::with_capacity(labels.len());
    for label in labels {
        let free = drawn.iter().all(|other| other.intersect(*label).is_empty());
        let shown = free && drawn.len() < MOST_LABELS;
        if shown {
            drawn.push(*label);
        }
        kept.push(shown);
    }
    kept
}

/// Returns the length in pixels and in metres of the scale bar, for a map where one pixel is
/// `metres_per_pixel` metres: the longest of 1, 2 or 5 times a power of ten that fits [`BAR`].
pub fn scale_bar(metres_per_pixel: f64) -> (f32, f64) {
    let most = metres_per_pixel * BAR;
    let power = 10f64.powi(most.log10().floor() as i32);
    let steps = [5.0, 2.0, 1.0].map(|step| step * power);
    let metres = steps
        .into_iter()
        .find(|step| *step <= most)
        .unwrap_or(power);
    ((metres / metres_per_pixel) as f32, metres)
}
