//! The fit of the game image to the window: a whole scale, about the same rows of game pixels
//! on every screen, and a crop of less than one game pixel.

use bevy::prelude::*;

use super::lowres::Fit;

/// Common windows, in physical pixels, get a whole scale and about 360 rows of game pixels.
#[test]
fn common_windows_show_about_the_same_rows() {
    let cases = [
        ((1280, 720), 2, (640, 360)),
        ((1920, 1080), 3, (640, 360)),
        ((2560, 1600), 4, (640, 400)),
        ((2880, 1800), 5, (576, 360)),
        ((3440, 1440), 4, (860, 360)),
        ((3840, 2160), 6, (640, 360)),
    ];
    for ((w, h), scale, (x, y)) in cases {
        let fit = Fit::of(UVec2::new(w, h));
        assert_eq!((fit.scale, fit.size), (scale, UVec2::new(x, y)), "{w}x{h}");
    }
}

/// A window that is not a multiple of the scale loses fewer than one game pixel of the image,
/// split evenly, and shows exactly `scale` window pixels to a game pixel.
#[test]
fn the_crop_stays_under_one_game_pixel() {
    for w in (480..4000).step_by(7) {
        for h in (270..2400).step_by(11) {
            let physical = UVec2::new(w, h);
            let fit = Fit::of(physical);
            let overflow = fit.size * fit.scale - physical;
            assert!(overflow.max_element() < fit.scale, "{w}x{h}: {fit:?}");
            assert!(
                (overflow - fit.crop * 2).max_element() <= 1,
                "{w}x{h}: {fit:?}"
            );
            let shown = fit.shown(physical);
            assert!(
                shown.max.cmple(fit.size.as_vec2() + 1e-3).all(),
                "{w}x{h}: {shown:?}"
            );
            let stretch = shown.size() * fit.scale as f32 - physical.as_vec2();
            assert!(stretch.abs().max_element() < 1e-3, "{w}x{h}: {shown:?}");
        }
    }
}

/// A tiny or empty window still gets an image big enough for the mip chains of the effects.
#[test]
fn tiny_windows_keep_a_usable_image() {
    assert_eq!(Fit::of(UVec2::ZERO).size, UVec2::splat(32));
    assert_eq!(Fit::of(UVec2::new(100, 20)).scale, 1);
}
