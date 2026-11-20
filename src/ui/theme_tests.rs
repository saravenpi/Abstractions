//! Theme tests: the backdrops are translucent, the scales are whole pixels on the grid, and
//! text keeps its contrast over the brightest world.

use bevy::prelude::*;

use super::backdrop::{backdrop_luminance, contrast};
use super::theme;

/// Converts one sRGB channel to linear light.
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Returns the relative luminance of an opaque sRGB colour.
fn luminance(c: Srgba) -> f32 {
    0.2126 * linear(c.red) + 0.7152 * linear(c.green) + 0.0722 * linear(c.blue)
}

/// Lays `top` over the opaque colour `under`, as the screen does, channel by channel in sRGB.
fn over(top: Color, under: Srgba) -> Srgba {
    let t = top.to_srgba();
    let mix = |a: f32, b: f32| a * t.alpha + b * (1.0 - t.alpha);
    Srgba::new(
        mix(t.red, under.red),
        mix(t.green, under.green),
        mix(t.blue, under.blue),
        1.0,
    )
}

/// Contrast of `text` drawn on `PANEL` over a world of the given brightness.
fn on_panel(text: Color, world: f32) -> f32 {
    let sky = Srgba::new(world, world, world, 1.0);
    let panel = over(theme::PANEL, sky);
    contrast(luminance(over(text, panel)), luminance(panel))
}

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

/// Text on `PANEL` stays readable over a pure white sky, the worst case, and over black.
#[test]
fn text_on_the_panel_is_readable_over_a_white_sky() {
    assert!(on_panel(theme::TEXT, 1.0) >= 7.0, "{}", on_panel(theme::TEXT, 1.0));
    assert!(on_panel(theme::TEXT_2, 1.0) >= 4.5, "{}", on_panel(theme::TEXT_2, 1.0));
    assert!(on_panel(theme::ACCENT, 1.0) >= 4.5, "{}", on_panel(theme::ACCENT, 1.0));
    assert!(on_panel(theme::PIN, 1.0) >= 4.5, "{}", on_panel(theme::PIN, 1.0));
    for warning in [theme::WARN, theme::DANGER] {
        assert!(on_panel(warning, 1.0) >= 3.5, "{warning:?} {}", on_panel(warning, 1.0));
        assert!(on_panel(warning, 0.0) >= 7.0);
    }
    assert!(on_panel(theme::TEXT, 0.0) >= 15.0);
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
    let (a, b, c) = (theme::TEXT.alpha(), theme::TEXT_2.alpha(), theme::TEXT_OFF.alpha());
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
