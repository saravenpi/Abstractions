//! Tests of the frame corner: a whole body shows nothing, a hurt one lists its worst part first.

use super::frame::lines;
use crate::condition::{Condition, Part};

/// A whole body has no lines.
#[test]
fn a_whole_body_shows_nothing() {
    assert!(lines(&Condition::default()).is_empty());
}

/// A hurt body shows its integrity and one line per hurt part, the worst first.
#[test]
fn hurt_parts_are_listed_worst_first() {
    let mut body = Condition::default();
    body.hurt(Part::LegL, 300.0, true);
    body.hurt(Part::Head, 900.0, true);
    let shown = lines(&body);
    assert_eq!(shown.len(), 3);
    assert!(shown[0].starts_with("Frame "));
    assert_eq!(shown[1], "Head 0%");
    assert!(shown[2].starts_with("Left leg "));
}
