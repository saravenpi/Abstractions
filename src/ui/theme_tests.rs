//! Theme tests: the interface has no hue, its backdrops are not opaque, and white text keeps
//! its contrast over the brightest world.

use bevy::prelude::*;

use super::backdrop::{backdrop_luminance, contrast};
use super::theme;

/// Every colour of the interface is neutral: its red, green and blue are equal.
#[test]
fn every_interface_colour_is_neutral() {
    for color in theme::PALETTE {
        let c = color.to_srgba();
        assert!(c.red == c.green && c.green == c.blue, "{color:?}");
    }
}

/// What sits over the game lets it show: the veil, the HUD strip and the selected block are
/// translucent. Text and the block of the current row are solid.
#[test]
fn backdrops_over_the_game_are_not_opaque() {
    for color in [theme::SCRIM, theme::STRIP, theme::SELECTED] {
        let alpha = color.alpha();
        assert!(alpha > 0.0 && alpha < 1.0, "{color:?}");
    }
    assert_eq!(theme::SCRIM.alpha(), theme::TINT);
    assert_eq!((theme::TEXT.alpha(), theme::BLOCK.alpha()), (1.0, 1.0));
    assert_eq!(theme::INK.to_srgba().red, 0.0);
}

/// White text over the darkened backdrop keeps a contrast of 4.5 to 1 even when the world
/// behind is pure white, and black text on the white block of the current row has the most.
#[test]
fn white_text_keeps_its_contrast_over_a_white_world() {
    let worst = backdrop_luminance(1.0, theme::TINT);
    assert!(contrast(1.0, worst) >= 4.5, "{}", contrast(1.0, worst));
    assert!(contrast(1.0, backdrop_luminance(0.0, theme::TINT)) > 20.0);
    assert!((contrast(0.0, 1.0) - 21.0).abs() < 1e-3);
    let tint = std::hint::black_box(theme::TINT);
    assert!(tint < 0.9, "the game must still show through");
}

/// Sizes that must stay on the pixel grid are whole numbers, and icons use their 24 pixel grid.
#[test]
fn sizes_are_whole_pixels() {
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
    assert_eq!(theme::ICON, 24.0);
}
