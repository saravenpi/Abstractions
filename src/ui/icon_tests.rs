//! Font tests: every icon has a glyph in the embedded icon font, and the embedded
//! interface font is an OpenType file with the characters the game writes.

use std::collections::HashSet;

use bevy::prelude::*;

use super::font::UNDEFINED_OTF;
use super::font_rig::{be16, glyph_index, table};
use super::icon::{CODEPOINTS, ICONS_TTF, Icon};
use super::theme;

/// Every icon maps to a glyph of the embedded font, and no two icons share a codepoint.
#[test]
fn every_icon_has_a_glyph_in_the_embedded_font() {
    assert_eq!(&ICONS_TTF[..4], [0, 1, 0, 0]);
    for (at, icon) in Icon::ALL.into_iter().enumerate() {
        assert_eq!(
            CODEPOINTS[at].0, icon,
            "the table follows the declaration order"
        );
        let index = glyph_index(ICONS_TTF, icon.codepoint());
        assert!(index > 0, "{icon:?} has no glyph");
        assert_eq!(icon.glyph().chars().count(), 1);
    }
    let codes: HashSet<char> = Icon::ALL.iter().map(|icon| icon.codepoint()).collect();
    assert_eq!(codes.len(), Icon::ALL.len());
    assert_eq!(
        glyph_index(ICONS_TTF, 'A'),
        0,
        "the parser finds glyphs everywhere"
    );
}

/// The interface font is an OpenType file Bevy takes as a font asset, it has a glyph for every
/// printable ASCII character, and its glyphs all advance by the width the layouts count on.
#[test]
fn the_interface_font_loads_and_covers_ascii() {
    assert_eq!(&UNDEFINED_OTF[..4], b"OTTO");
    let font = Font::from_bytes(UNDEFINED_OTF.to_vec());
    assert_eq!(font.data.len(), UNDEFINED_OTF.len());
    for c in '!'..='~' {
        assert!(glyph_index(UNDEFINED_OTF, c) > 0, "no glyph for {c:?}");
    }
    let units = be16(UNDEFINED_OTF, table(UNDEFINED_OTF, b"head") + 18) as f32;
    let advance = be16(UNDEFINED_OTF, table(UNDEFINED_OTF, b"hmtx")) as f32;
    assert_eq!(advance / units, theme::GLYPH_ASPECT);
}
