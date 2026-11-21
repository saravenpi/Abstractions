//! Stacking the marks of the compass: the words of a mark, the width of its strip, and the rows
//! that keep marks which would overlap apart.

use std::fmt::Write;

use super::ROWS;
use super::show::{Seen, kind_name};
use crate::computer::distance;
use crate::menus::Marker;
use crate::menus::hud::needle::WIDTH;
use crate::ui::theme;

/// Pixels between two marks of one row.
const APART: f32 = theme::S2;

/// Writes the words of `marker` at `gap` metres into `out`: its name, or its kind, and its distance.
pub(super) fn phrase(marker: &Marker, gap: f64, out: &mut String) {
    out.clear();
    let name = match marker.label.is_empty() {
        true => kind_name(marker.kind),
        false => marker.label.as_str(),
    };
    let _ = write!(out, "{name}  {}", distance(gap));
}

/// Returns the width in pixels of the strip of a mark whose words are `words`.
pub(super) fn width(words: &str) -> f32 {
    let glyph = theme::GLYPH_ASPECT * theme::FONT_SMALL;
    words.chars().count() as f32 * glyph + theme::ICON_SMALL + theme::S1 + 2.0 * theme::S2
}

/// Gives each of `seen` its row, nearest first, and the place `at` that keeps its strip on the
/// bar. A mark that finds no row with room is dropped.
pub(super) fn stack(seen: &mut Vec<Seen>) {
    seen.sort_by(|a, b| a.gap.total_cmp(&b.gap));
    let mut taken: [Vec<(f32, f32)>; ROWS] = Default::default();
    seen.retain_mut(|mark| {
        let reach = mark.wide * 0.5;
        mark.at = mark.at.clamp(reach, WIDTH - reach);
        let span = (mark.at - reach - APART, mark.at + reach + APART);
        let free = |row: &Vec<(f32, f32)>| row.iter().all(|t| span.1 <= t.0 || span.0 >= t.1);
        let Some(row) = taken.iter().position(free) else {
            return false;
        };
        taken[row].push(span);
        mark.row = row;
        true
    });
}
