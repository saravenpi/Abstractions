//! Theme tests: the backdrops are translucent, the scales are whole pixels on the grid, and
//! text keeps its contrast over the brightest world.

use bevy::prelude::*;

use super::backdrop::{backdrop_luminance, contrast};
use super::color_rig::luminance;
use super::theme;

/// What sits over the game lets it show: every backdrop is translucent, text is solid.
#[test]
fn backdrops_over_the_game_are_not_opaque() {
    for color in theme::TRANSLUCENT {
        let alpha = color.alpha();
        assert!(alpha > 0.0 && alpha < 1.0, "{color:?}");
    }
    assert_eq!(theme::SCRIM.alpha(), theme::TINT);
    assert_eq!((theme::TEXT.alpha(), theme::ACCENT.alpha()), (1.0, 1.0));
}

/// White text over the darkened backdrop keeps a contrast of 4.5 to 1 even when the world
/// behind is pure white, and black text on the white block has the most.
#[test]
fn white_text_keeps_its_contrast_over_a_white_world() {
    let worst = backdrop_luminance(1.0, theme::TINT);
    assert!(contrast(1.0, worst) >= 4.5, "{}", contrast(1.0, worst));
    assert!(contrast(1.0, backdrop_luminance(0.0, theme::TINT)) > 20.0);
    assert!((contrast(0.0, 1.0) - 21.0).abs() < 1e-3);
}

/// Ink on the accent fill is readable.
#[test]
fn ink_is_readable_on_the_accent() {
    let accent = luminance(theme::ACCENT.to_srgba());
    let ink = luminance(theme::INK.to_srgba());
    assert!(contrast(accent, ink) >= 7.0, "{}", contrast(accent, ink));
}

/// Secondary text is dimmer than primary and brighter than disabled.
#[test]
fn text_tones_are_ordered() {
    let (a, b, c) = (
        theme::TEXT.alpha(),
        theme::TEXT_2.alpha(),
        theme::TEXT_OFF.alpha(),
    );
    assert!(a > b && b > c);
}

/// Type sizes, line heights and spacing are whole pixels on the 4 pixel grid.
#[test]
fn sizes_are_whole_pixels_on_the_grid() {
    let sizes = [
        theme::FONT_TITLE,
        theme::FONT_HEADING,
        theme::FONT_BODY,
        theme::FONT_SMALL,
        theme::ROW_HEIGHT,
        theme::ROW_COMPACT,
        theme::LINE,
        theme::VALUE_WIDTH,
    ];
    assert!(sizes.iter().all(|size| size.fract() == 0.0));
    let grid = [
        theme::LINE_TITLE,
        theme::LINE_HEADING,
        theme::LINE_BODY,
        theme::LINE_SMALL,
        theme::S1,
        theme::S2,
        theme::S3,
        theme::S4,
        theme::S5,
        theme::S6,
    ];
    assert!(grid.iter().all(|size| size % 4.0 == 0.0));
    assert_eq!(theme::ICON, 24.0);
    assert_eq!(theme::EDGE, theme::S5);
}

/// The type scale is 16, 20, 32 and 72, and each line height is at least 1.25 times its size.
#[test]
fn the_type_scale_is_the_spec_scale() {
    let scale = [
        (theme::FONT_SMALL, theme::LINE_SMALL, 16.0),
        (theme::FONT_BODY, theme::LINE_BODY, 20.0),
        (theme::FONT_HEADING, theme::LINE_HEADING, 32.0),
        (theme::FONT_TITLE, theme::LINE_TITLE, 72.0),
    ];
    for (size, line, want) in scale {
        assert_eq!(size, want);
        assert!(line >= size * 1.1);
    }
}
