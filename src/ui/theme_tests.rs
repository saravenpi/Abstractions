//! Theme tests: the backdrops are translucent, the scales are whole pixels on the grid, and
//! text keeps its contrast over the brightest world.

use bevy::prelude::*;

use super::backdrop::{backdrop_luminance, contrast};
use super::color_rig::{holds, luminance, on_fill, on_panel};
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

/// Text on the light `PANEL`, over a pure white sky, the worst case no tonemapper reaches: `TEXT`
/// keeps 5.5 to 1, the secondary text 3.5 to 1, and the pin and accent colours 4 and 3 to 1.
#[test]
fn text_on_the_panel_is_readable_over_a_white_sky() {
    let (t, t2, pin, accent) = (theme::TEXT, theme::TEXT_2, theme::PIN, theme::ACCENT);
    holds(1.0, &[(t, 5.5), (t2, 3.5), (pin, 4.0), (accent, 3.0)]);
}

/// Over a bright sky of 0.8, which the game's tonemapper does not exceed in the shots taken so far
/// (an assumption to check on new shots), the secondary text reaches 4.5 to 1. Over black every
/// text keeps 7 to 1 or more.
#[test]
fn text_on_the_panel_is_readable_over_a_bright_sky_and_black() {
    let (t, t2, pin, accent) = (theme::TEXT, theme::TEXT_2, theme::PIN, theme::ACCENT);
    holds(0.8, &[(t2, 4.5), (pin, 5.0), (accent, 4.0)]);
    holds(0.0, &[(t, 15.0), (t2, 7.0), (pin, 7.0), (accent, 7.0)]);
}

/// The rows that can hold warning text sit on `PANEL_DEEP`, where `WARN` and `DANGER` keep 4.5 to 1
/// over a white sky and 7 to 1 over black. They do not reach 3 to 1 on the light `PANEL`, which is
/// why those rows have the deeper fill.
#[test]
fn warning_text_is_readable_on_the_deep_panel() {
    for warning in [theme::WARN, theme::DANGER] {
        let white = on_fill(warning, theme::PANEL_DEEP, 1.0);
        assert!(white >= 4.5, "{warning:?} {white}");
        assert!(on_fill(warning, theme::PANEL_DEEP, 0.0) >= 7.0);
    }
    assert!(on_panel(theme::DANGER, 1.0) < 3.0);
    assert!(theme::PANEL.alpha() < theme::PANEL_DEEP.alpha());
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
