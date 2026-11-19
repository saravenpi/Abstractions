//! Tests of the end of a course: the arrival shows for a moment, then the course ends.

use super::done::{SHOWN, tick};

/// The course ends once the route has read as arrived for the whole time it shows.
#[test]
fn an_arrival_ends_the_course_after_it_shows() {
    let (mut since, mut done) = (0.0, false);
    let mut frames = 0;
    while !done {
        (since, done) = tick(since, 0.1, true);
        frames += 1;
    }
    assert!((frames as f32 * 0.1 - SHOWN).abs() < 0.11, "{frames}");
}

/// A route that leaves the arrived state starts counting again from zero.
#[test]
fn leaving_the_arrival_resets_the_count() {
    let (since, done) = tick(SHOWN - 0.5, 0.1, false);
    assert_eq!((since, done), (0.0, false));
    assert!(!tick(since, 1.0, true).1);
}
