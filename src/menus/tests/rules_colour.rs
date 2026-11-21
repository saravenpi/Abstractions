//! The colour rule, held on every converted page: every fill and every text colour is a token
//! of the theme, each with its one meaning.

use bevy::prelude::*;

use super::rules::each_page;
use crate::ui::theme;

/// Returns `color` as its four sRGB channels, so that two ways to write one colour compare equal.
fn channels(color: Color) -> [f32; 4] {
    let color = color.to_srgba();
    [color.red, color.green, color.blue, color.alpha]
}

/// The fills a page may use.
const FILLS: [Color; 8] = [
    Color::NONE,
    theme::SCRIM,
    theme::PANEL,
    theme::PANEL_BLUR,
    theme::HOVER,
    theme::SELECT,
    theme::TEXT_2,
    theme::HAIRLINE,
];

/// The colours a text of a page may have.
const TONES: [Color; 7] = [
    Color::NONE,
    theme::TEXT,
    theme::TEXT_2,
    theme::TEXT_OFF,
    theme::ACCENT,
    theme::WARN,
    theme::DANGER,
];

/// Whether `color` is one of `tokens`.
fn known(tokens: &[Color], color: Color) -> bool {
    tokens
        .iter()
        .any(|token| channels(*token) == channels(color))
}

/// Every fill and every text colour is a token of the theme.
#[test]
fn every_colour_is_a_token() {
    each_page("tokens", |world, found, menu| {
        for entity in found {
            if let Some(fill) = world.get::<BackgroundColor>(*entity) {
                assert!(known(&FILLS, fill.0), "{menu:?}: fill {:?}", fill.0);
            }
            if let Some(tone) = world.get::<TextColor>(*entity) {
                assert!(known(&TONES, tone.0), "{menu:?}: text {:?}", tone.0);
            }
        }
    });
}
