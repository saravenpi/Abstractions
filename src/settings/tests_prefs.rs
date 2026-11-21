//! Tests that a change of the camera made in play reaches the settings file once, after it rests.

use super::prefs::{Kept, SETTLE};
use crate::view::ViewPrefs;

/// Seconds of one frame at 60 frames a second.
const FRAME: f32 = 1.0 / 60.0;

/// Returns how many writes three seconds of a wheel turned a little on every frame, then two
/// seconds of rest, ask for.
fn writes_of_a_long_turn(start: ViewPrefs) -> usize {
    let (mut kept, mut saved, mut now) = (Kept::default(), start, start);
    let mut writes = 0;
    for frame in 0..300 {
        if frame < 180 {
            now.zoom_by(-0.05);
        }
        if let Some(chosen) = kept.due(saved, now, FRAME) {
            saved = chosen;
            writes += 1;
        }
    }
    writes
}

/// A wheel turned for three seconds, as the glide of a trackpad does, is written once, after it
/// rests, with the zoom it ended at. The old keeper wrote it every 0.6 s while it turned.
#[test]
fn a_long_turn_of_the_wheel_is_one_write() {
    let start = ViewPrefs {
        zoom: 0.5,
        ..ViewPrefs::default()
    };
    assert_eq!(writes_of_a_long_turn(start), 1);
}

/// Preferences that do not change are never written, and a change is written only after it has
/// rested for the settle time.
#[test]
fn a_change_waits_for_the_settle_time() {
    let (mut kept, saved) = (Kept::default(), ViewPrefs::default());
    for _ in 0..120 {
        assert_eq!(kept.due(saved, saved, FRAME), None);
    }
    let mut now = saved;
    now.zoom_by(1.0);
    let frames = (SETTLE / FRAME).ceil() as usize;
    let early = (0..frames - 1)
        .filter_map(|_| kept.due(saved, now, FRAME))
        .count();
    assert_eq!(early, 0);
    let late = (0..4).filter_map(|_| kept.due(saved, now, FRAME)).next();
    assert_eq!(late, Some(now));
}

/// A NaN from the wheel leaves the zoom alone, and a NaN zoom is never written to the file.
#[test]
fn a_nan_zoom_is_never_written() {
    let mut prefs = ViewPrefs::default();
    prefs.zoom_by(f32::NAN);
    assert_eq!(prefs, ViewPrefs::default());
    let broken = ViewPrefs {
        zoom: f32::NAN,
        ..ViewPrefs::default()
    };
    let mut kept = Kept::default();
    let writes = (0..600).filter_map(|_| kept.due(ViewPrefs::default(), broken, FRAME));
    assert_eq!(writes.count(), 0);
    let mut healed = broken;
    healed.zoom_by(1.0);
    assert!(healed.zoom.is_finite());
}
