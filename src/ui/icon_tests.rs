//! Font tests: every icon has a glyph in the embedded pixelarticons font, and the embedded
//! interface font is an OpenType file with the characters the game writes.

use std::collections::HashSet;

use bevy::prelude::*;

use super::font::UNDEFINED_OTF;
use super::icon::{CODEPOINTS, ICONS_TTF, Icon};
use super::theme;

/// Reads a big-endian `u16` at `at`.
fn be16(data: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes([data[at], data[at + 1]]))
}

/// Reads a big-endian `u32` at `at`.
fn be32(data: &[u8], at: usize) -> usize {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]) as usize
}

/// Returns the offset of the table named `tag` in a font file.
fn table(data: &[u8], tag: &[u8; 4]) -> usize {
    let found = (0..be16(data, 4))
        .map(|i| 12 + 16 * i)
        .find(|at| &data[*at..*at + 4] == tag);
    be32(data, found.expect("the font has the table") + 8)
}

/// Returns the offset of the first format 12 subtable of the character map.
fn full_map(data: &[u8]) -> usize {
    let cmap = table(data, b"cmap");
    let found = (0..be16(data, cmap + 2))
        .map(|i| cmap + be32(data, cmap + 8 + 8 * i))
        .find(|at| be16(data, *at) == 12);
    found.expect("the font has a format 12 character map")
}

/// Returns the glyph index the font maps `c` to, or 0 when it has no glyph for it.
fn glyph_index(data: &[u8], c: char) -> usize {
    let (map, code) = (full_map(data), c as usize);
    let group = (0..be32(data, map + 12))
        .map(|i| map + 16 + 12 * i)
        .find(|at| (be32(data, *at)..=be32(data, *at + 4)).contains(&code));
    group.map_or(0, |at| be32(data, at + 8) + code - be32(data, at))
}

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

/// The icons of the multiplayer menus keep the codepoints of `pixelart-icons-font.css` 2.4.2,
/// and the embedded font draws each of them.
#[test]
fn the_multiplayer_icons_match_the_pinned_font() {
    let pinned = [
        (Icon::Users, 0xede1),
        (Icon::User, 0xeddf),
        (Icon::Server, 0xed20),
        (Icon::Wifi, 0xee01),
        (Icon::Reload, 0xecfa),
        (Icon::Link, 0xec5f),
    ];
    for (icon, code) in pinned {
        assert_eq!(icon.codepoint() as u32, code, "{icon:?}");
        assert!(glyph_index(ICONS_TTF, icon.codepoint()) > 0, "{icon:?}");
        assert!(Icon::ALL.contains(&icon), "{icon:?}");
    }
}
