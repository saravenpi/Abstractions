//! Tests of the measures of the map screen: the legend of the keys and the grid.

use super::{
    BETWEEN, EDGE, GLYPH, LEGEND, LEGEND_LINES, LINE_CHARS, LIST_ROW, MARGIN, PAD, ROOM, ROW,
    SECTION_GAP, SIDE,
};
use crate::ui::{HINT_GAP, KEY_GAP};

/// Returns the width in pixels of one hint of the legend: its label, the gap and its key.
fn hint((words, key): (&str, &str)) -> f32 {
    (words.chars().count() + key.chars().count()) as f32 * GLYPH + KEY_GAP
}

/// Returns how many lines the hints of `legend` take in a row `room` pixels wide that wraps,
/// with the gap of a legend between two hints of one line.
fn lines(legend: &[(&str, &str)], room: f32) -> usize {
    let (mut count, mut used) = (1, 0.0);
    for width in legend.iter().copied().map(hint) {
        if used > 0.0 && used + HINT_GAP + width > room {
            (count, used) = (count + 1, width);
        } else {
            used += [0.0, HINT_GAP][usize::from(used > 0.0)] + width;
        }
    }
    count
}

/// The legend of the keys fits the lines the panel keeps for it, each hint fits one line, and it
/// names the keys no control of the screen shows: Go, Home and the search carry their own.
#[test]
fn the_legend_fits_its_lines() {
    assert!(lines(&LEGEND, ROOM) <= LEGEND_LINES, "{LEGEND:?}");
    assert!(LEGEND.iter().all(|pair| hint(*pair) <= ROOM));
    let keys: Vec<&str> = LEGEND.iter().map(|(_, key)| *key).collect();
    assert_eq!(keys, ["Click", "B", "Wheel", "Esc"]);
    assert!(LEGEND.iter().all(|(words, _)| !words.contains('[')));
    assert_eq!(LINE_CHARS, (ROOM / GLYPH) as usize);
}

/// The measures of the screen sit on the grid of four pixels.
#[test]
fn the_measures_sit_on_the_grid() {
    for size in [SIDE, PAD, MARGIN, EDGE, BETWEEN, SECTION_GAP, LIST_ROW, ROW] {
        assert_eq!(size % 4.0, 0.0, "{size}");
    }
}
