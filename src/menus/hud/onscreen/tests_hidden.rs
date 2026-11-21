//! Tests of a marker whose place lies behind the planet: it is drawn at its place, fainter, and
//! its words say why, rather than hidden.

use super::behind;
use super::rule::Face;
use super::tune::{HIDDEN_ALPHA, HIDDEN_WORDS};

/// A whole marker, as the home base far away is drawn.
const WHOLE: Face = Face {
    glyph: 1.0,
    label: 1.0,
    small: false,
};

/// Behind the planet the home marker keeps its words and its place, its icon at the share of
/// [`HIDDEN_ALPHA`], and says it is behind the planet.
#[test]
fn the_home_marker_behind_the_planet_is_drawn_hollow() {
    let mut words = "Home 2,340 km".to_string();
    let face = behind(&mut words, WHOLE, true);
    assert_eq!(words, format!("Home 2,340 km{HIDDEN_WORDS}"));
    assert_eq!(face.glyph, HIDDEN_ALPHA);
    assert_eq!((face.label, face.small), (1.0, false));
}

/// In sight, a marker is drawn as its rules say, and a marker without words gets none.
#[test]
fn a_marker_in_sight_is_left_as_it_is() {
    let mut words = "Home 300 m".to_string();
    assert_eq!(behind(&mut words, WHOLE, false), WHOLE);
    assert_eq!(words, "Home 300 m");
    let mut none = String::new();
    let small = Face {
        label: 0.0,
        small: true,
        ..WHOLE
    };
    behind(&mut none, small, true);
    assert!(none.is_empty());
}
