//! Width tests: every text of the side panel fits its room at the smallest window, so none is
//! cut off at the right edge. The panel keeps its width at every window size. Chart names keep
//! out of the header and footer words.

use bevy::math::Vec2;

use super::format::clip;
use super::labels::room;
use super::layout::{GLYPH, LINE_CHARS, ROOM, SUMMARY_CHARS, clear_of_bands};
use super::results::{DISTANCE_CHARS, TITLE_CHARS, WIDTHS};
use super::show::cell_text;
use super::sift::{FILTER_CHARS, Sort, summary};
use super::view::Part;
use super::{Line, Target};
use crate::ui::theme;
use crate::ui::units::distance_short;
use crate::universe::{BodyId, LIGHT_YEAR};

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
    assert_eq!(clip("Kolva", TITLE_CHARS), "Kolva");
}

/// Every distance from a metre to a billion light-years fits the distance column, short.
#[test]
fn every_distance_fits_its_column() {
    let mut metres = 1.0;
    while metres < 1e9 * LIGHT_YEAR {
        let shown = distance_short(metres);
        assert!(shown.chars().count() <= DISTANCE_CHARS, "{shown}");
        metres *= 1.37;
    }
    assert!(distance_short(102_007.0 * LIGHT_YEAR).chars().count() <= DISTANCE_CHARS);
}

/// The line that counts the results fits beside the heading of the search, whatever the
/// heading, the filter, the count and the order.
#[test]
fn the_header_fits_one_line() {
    let caps = "Search".len() as f32 * (GLYPH + theme::CAPS_SPACING);
    assert!(caps + theme::GAP_S + SUMMARY_CHARS as f32 * GLYPH <= ROOM);
    let filter = "x".repeat(FILTER_CHARS);
    let headings = ["Varena system", "Known universe", LONGEST];
    for heading in headings {
        for sort in [Sort::Near, Sort::Name] {
            for (count, typed) in [(0, ""), (0, filter.as_str()), (999, ""), (999, &filter)] {
                let shown = summary(heading, (991, 999.min(count), count), (typed, sort));
                assert!(shown.chars().count() <= SUMMARY_CHARS, "{shown}");
            }
        }
    }
    let usual = summary("Varena system", (1, 8, 12), ("", Sort::Near));
    assert_eq!(usual, "Varena system - 1-8 of 12");
}

/// A chart name stays clear of the header and footer words: one in the middle of the map is
/// drawn, one under the path or beside the progress line is not.
#[test]
fn chart_names_keep_out_of_the_header_and_footer() {
    let label = |y: f32| room(Vec2::new(200.0, y), 10.0, 6);
    assert!(clear_of_bands(label(360.0), 720.0));
    assert!(!clear_of_bands(label(22.0), 720.0));
    assert!(!clear_of_bands(label(644.0), 720.0));
}
