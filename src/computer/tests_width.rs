//! Width tests: every text of the side panel fits its room at the smallest window, so none is
//! cut off at the right edge. The panel keeps its width at every window size.

use super::format::{clip, distance};
use super::layout::{GLYPH, LINE_CHARS, ROOM};
use super::results::{DISTANCE_CHARS, TITLE_CHARS, WIDTHS};
use super::show::cell_text;
use super::sift::{FILTER_CHARS, Sort, summary};
use super::view::{HINT, Part};
use super::{Line, Target};
use crate::ui::theme;
use crate::universe::{BodyId, GLY};

/// The longest name a row can carry: a Greek prefix, a long made-up word, a moon numeral and
/// the home label.
const LONGEST: &str = "Epsilon Thraekrorthar 99 b IV (Home)";

/// Returns a moon row named `title`, `metres` away.
fn moon(title: &str, metres: f64) -> Line {
    Line {
        title: title.to_string(),
        detail: String::new(),
        target: Target::Body(BodyId {
            moon: 3,
            ..BodyId::default()
        }),
        distance: metres,
        visited: true,
        at: Default::default(),
    }
}

/// The icon, name and distance columns of a result fit the room of the panel with their gaps,
/// and each column holds its character budget.
#[test]
fn a_result_row_fits_the_panel() {
    let row = WIDTHS.iter().sum::<f32>() + 2.0 * theme::GAP_XS;
    assert!(row <= ROOM, "{row} > {ROOM}");
    assert!(TITLE_CHARS as f32 * GLYPH <= WIDTHS[1]);
    assert!(LINE_CHARS as f32 * GLYPH <= ROOM);
    let long = moon(LONGEST, 0.0);
    let title = cell_text(&long, Part::Title, (false, false)).0;
    assert!(title.chars().count() <= TITLE_CHARS, "{title}");
    assert!(title.ends_with(".."));
    assert_eq!(clip("Kodya", TITLE_CHARS), "Kodya");
}

/// Every distance from a metre to a million game light-years fits the distance column.
#[test]
fn every_distance_fits_its_column() {
    let mut metres = 1.0;
    while metres < 1e6 * GLY {
        let shown = distance(metres);
        assert!(shown.chars().count() <= DISTANCE_CHARS, "{shown}");
        metres *= 1.37;
    }
    assert!(distance(102_007.0 * GLY).chars().count() <= DISTANCE_CHARS);
}

/// The header over the results stays on one line of the panel, whatever the heading, the
/// filter, the count and the order.
#[test]
fn the_header_fits_one_line() {
    let filter = "x".repeat(FILTER_CHARS);
    let headings = ["Vereya system", "Known universe", LONGEST];
    for heading in headings {
        for sort in [Sort::Near, Sort::Name] {
            for (count, typed) in [(0, ""), (0, filter.as_str()), (999, ""), (999, &filter)] {
                let shown = summary(heading, (991, 999.min(count), count), (typed, sort));
                assert!(shown.chars().count() <= LINE_CHARS, "{shown}");
            }
        }
    }
    let usual = summary("Vereya system", (1, 8, 12), ("", Sort::Near));
    assert_eq!(usual, "Vereya system - 1-8 of 12");
}

/// Each line of the key hints fits the panel, and the hints take two lines.
#[test]
fn the_hint_fits_the_panel() {
    assert_eq!(HINT.lines().count(), 2);
    for line in HINT.lines() {
        assert!(line.chars().count() <= LINE_CHARS, "{line}");
    }
}
