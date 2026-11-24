//! The fit of the game image to the window: capped at 540 rows on larger windows, monotonic on
//! smaller ones, and matching the window aspect ratio without cropping.

use bevy::prelude::*;

use super::lowres::Fit;
use super::tune::REFERENCE_ROWS;

/// Common windows, in physical pixels, cap height at 540 rows and keep their exact aspect ratio.
#[test]
fn common_windows_keep_consistent_rows_and_aspect() {
    let cases = [
        ((960, 540), (960, 540)),
        ((1280, 720), (960, 540)),
        ((1920, 1080), (960, 540)),
        ((2560, 1600), (864, 540)),
        ((2880, 1800), (864, 540)),
        ((3440, 1440), (1290, 540)),
        ((3840, 2160), (960, 540)),
    ];
    for ((w, h), (x, y)) in cases {
        let fit = Fit::of(UVec2::new(w, h));
        assert_eq!(fit.size, UVec2::new(x, y), "{w}x{h}");
    }
}

/// Detail never drops when enlarging a window and always matches the window aspect ratio within a
/// single game pixel.
#[test]
fn detail_is_monotonic_and_aspect_matches_window() {
    let mut prev_rows = 0;
    for h in (270..=2400).step_by(11) {
        let rows = Fit::of(UVec2::new(1920, h)).size.y;
        assert!(rows >= prev_rows, "{h}: {rows} < {prev_rows}");
        if h >= REFERENCE_ROWS {
            assert_eq!(rows, REFERENCE_ROWS, "{h}: {rows}");
        }
        prev_rows = rows;
    }
    for w in (480..4000).step_by(71) {
        for h in (270..2400).step_by(43) {
            let physical = UVec2::new(w, h);
            let fit = Fit::of(physical);
            let want = w as f32 / h as f32;
            let got = fit.size.x as f32 / fit.size.y as f32;
            assert!(
                (got - want).abs() <= 1.0 / fit.size.y as f32,
                "{w}x{h}: {fit:?}"
            );
        }
    }
}

/// A tiny or empty window still gets an image big enough for the mip chains of the effects.
#[test]
fn tiny_windows_keep_a_usable_image() {
    assert_eq!(Fit::of(UVec2::ZERO).size, UVec2::splat(32));
    assert_eq!(Fit::of(UVec2::new(100, 20)).size.y, 32);
}
