//! Test rig: the width the interface font gives a line of text, read from the font file. Each
//! glyph advances by its horizontal metric plus what the single adjustments of the `kern`
//! feature add to it, as the text shaper does.

use super::font::UNDEFINED_OTF;
use super::font_rig::{be16, glyph_index, table};

/// Returns the advance of `glyph` in font units, from the horizontal metrics.
fn advance(data: &[u8], glyph: usize) -> usize {
    let metrics = be16(data, table(data, b"hhea") + 34);
    be16(data, table(data, b"hmtx") + 4 * glyph.min(metrics - 1))
}

/// Returns the place of `glyph` in the coverage table at `at`, or nothing when it is not covered.
fn covered(data: &[u8], at: usize, glyph: usize) -> Option<usize> {
    let count = be16(data, at + 2);
    if be16(data, at) == 1 {
        return (0..count).find(|i| be16(data, at + 4 + 2 * i) == glyph);
    }
    (0..count)
        .map(|i| at + 4 + 6 * i)
        .find(|range| (be16(data, *range)..=be16(data, *range + 2)).contains(&glyph))
        .map(|range| be16(data, range + 4) + glyph - be16(data, range))
}

/// Returns the advance the single adjustment subtable at `sub` adds to `glyph`, in font units.
fn single(data: &[u8], sub: usize, glyph: usize) -> i32 {
    let format = be16(data, sub + 4);
    let Some(place) = covered(data, sub + be16(data, sub + 2), glyph).filter(|_| format & 4 != 0)
    else {
        return 0;
    };
    let before = 2 * (format & 3).count_ones() as usize;
    let record = 2 * format.count_ones() as usize;
    let at = match be16(data, sub) {
        1 => sub + 6 + before,
        _ => sub + 8 + place * record + before,
    };
    i32::from(be16(data, at) as u16 as i16)
}

/// Returns the advance every single adjustment of the font adds to `glyph`, in font units.
fn adjustment(data: &[u8], glyph: usize) -> i32 {
    let gpos = table(data, b"GPOS");
    let list = gpos + be16(data, gpos + 8);
    (0..be16(data, list))
        .map(|i| list + be16(data, list + 2 + 2 * i))
        .filter(|lookup| be16(data, *lookup) == 1)
        .flat_map(move |lookup| {
            (0..be16(data, lookup + 4)).map(move |i| lookup + be16(data, lookup + 6 + 2 * i))
        })
        .map(|sub| single(data, sub, glyph))
        .sum()
}

/// Returns the width in logical pixels of `line` in the interface font at `size` pixels, with
/// `spacing` pixels after every glyph. A character the font has no glyph for fails the test:
/// the game would draw it in another font.
pub fn text_width(line: &str, size: f32, spacing: f32) -> f32 {
    let data = UNDEFINED_OTF;
    let units = be16(data, table(data, b"head") + 18) as f32;
    let glyph = |c: char| {
        let found = glyph_index(data, c);
        assert!(
            found > 0,
            "the interface font has no glyph for {c:?} in {line:?}"
        );
        found
    };
    line.chars()
        .map(glyph)
        .map(|found| {
            (advance(data, found) as f32 + adjustment(data, found) as f32) * size / units + spacing
        })
        .sum()
}
