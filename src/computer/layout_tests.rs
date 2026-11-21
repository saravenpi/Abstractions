//! Tests of the measures of the map screen: the lists of the panel, the legend and the grid.

use bevy::math::Vec2;

use super::{
    GLYPH, LEGEND, LEGEND_LINES, LINE_CHARS, LIST_ROW, MARGIN, PAD, ROOM, ROW, SECTION_GAP, SIDE,
    listed, measure,
};
use crate::map::PINS;

/// The smallest window, 960 by 540 pixels, is laid out as 1280 by 720 once the interface scales
/// by 0.75, and there the card keeps the room of its compact form, so no pin is listed. A tall
/// window lists all eight pins.
#[test]
fn the_panel_lists_the_pins_the_card_leaves_room_for() {
    let logical = 540.0 / 0.75;
    assert_eq!(PINS, 8);
    assert_eq!(listed(logical, (PINS, 0)), (0, 0));
    assert_eq!(listed(1080.0, (PINS, 0)), (PINS, 0));
    assert_eq!(listed(1080.0, (3, 0)), (3, 0));
    assert_eq!(
        measure(Vec2::new(2560.0, 1440.0), (PINS, 0)).lists,
        (PINS, 0)
    );
}

/// A window too short for the pins lists the rows it has room for, and never a part of a row.
#[test]
fn a_short_window_lists_fewer_pins() {
    let (rows, _) = listed(600.0, (PINS, 0));
    assert!(rows < PINS);
    let (none, _) = listed(300.0, (PINS, 0));
    assert_eq!(none, 0);
}

/// Wraps `words` greedily on spaces into lines of at most `chars` characters, keeping the
/// double spaces, and returns the lines.
fn wrapped(words: &str, chars: usize) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in words.split(' ') {
        let line = lines.last_mut().unwrap();
        let used = line.chars().count();
        if used > 0 && used + 1 + word.chars().count() > chars {
            lines.push(word.to_string());
        } else if used > 0 {
            line.push(' ');
            line.push_str(word);
        } else {
            line.push_str(word);
        }
    }
    lines
}

/// The legend of the keys fits the lines the panel keeps for it, and names every key of the
/// table of keys of the map.
#[test]
fn the_legend_fits_two_lines() {
    assert!(
        wrapped(LEGEND, LINE_CHARS).len() <= LEGEND_LINES,
        "{LEGEND}"
    );
    for key in [
        "Click select",
        "G go",
        "B pin",
        "H home",
        "/ search",
        "Esc close",
    ] {
        assert!(LEGEND.contains(key), "{key}");
    }
    assert_eq!(LINE_CHARS, (ROOM / GLYPH) as usize);
}

/// The measures of the screen sit on the grid of four pixels.
#[test]
fn the_measures_sit_on_the_grid() {
    for size in [SIDE, PAD, MARGIN, SECTION_GAP, LIST_ROW, ROW] {
        assert_eq!(size % 4.0, 0.0, "{size}");
    }
}
