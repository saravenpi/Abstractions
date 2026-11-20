//! Test rig: a small reader of the tables of a font file, enough to find the glyph a font gives a
//! character.

/// Reads a big-endian `u16` at `at`.
pub fn be16(data: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes([data[at], data[at + 1]]))
}

/// Reads a big-endian `u32` at `at`.
pub fn be32(data: &[u8], at: usize) -> usize {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]) as usize
}

/// Returns the offset of the table named `tag` in a font file.
pub fn table(data: &[u8], tag: &[u8; 4]) -> usize {
    let found = (0..be16(data, 4))
        .map(|i| 12 + 16 * i)
        .find(|at| &data[*at..*at + 4] == tag);
    be32(data, found.expect("the font has the table") + 8)
}

/// Returns the offset of the first character map subtable of `format`, if the font has one.
pub fn subtable(data: &[u8], format: usize) -> Option<usize> {
    let cmap = table(data, b"cmap");
    (0..be16(data, cmap + 2))
        .map(|i| cmap + be32(data, cmap + 8 + 8 * i))
        .find(|at| be16(data, *at) == format)
}

/// Returns the glyph a format 12 subtable at `map` gives `code`, or 0.
pub fn glyph_in_12(data: &[u8], map: usize, code: usize) -> usize {
    let group = (0..be32(data, map + 12))
        .map(|i| map + 16 + 12 * i)
        .find(|at| (be32(data, *at)..=be32(data, *at + 4)).contains(&code));
    group.map_or(0, |at| be32(data, at + 8) + code - be32(data, at))
}

/// Returns the glyph a format 4 subtable at `map` gives `code`, or 0.
pub fn glyph_in_4(data: &[u8], map: usize, code: usize) -> usize {
    let segments = be16(data, map + 6) / 2;
    let ends = map + 14;
    let starts = ends + 2 * segments + 2;
    let deltas = starts + 2 * segments;
    let ranges = deltas + 2 * segments;
    let Some(i) = (0..segments).find(|i| be16(data, ends + 2 * i) >= code) else {
        return 0;
    };
    if be16(data, starts + 2 * i) > code {
        return 0;
    }
    let (delta, range) = (be16(data, deltas + 2 * i), be16(data, ranges + 2 * i));
    if range == 0 {
        return (code + delta) & 0xffff;
    }
    let at = ranges + 2 * i + range + 2 * (code - be16(data, starts + 2 * i));
    match be16(data, at) {
        0 => 0,
        glyph => (glyph + delta) & 0xffff,
    }
}

/// Returns the glyph index the font maps `c` to, or 0 when it has no glyph for it.
pub fn glyph_index(data: &[u8], c: char) -> usize {
    let code = c as usize;
    if let Some(map) = subtable(data, 12) {
        return glyph_in_12(data, map, code);
    }
    let map = subtable(data, 4).expect("the font has a character map");
    glyph_in_4(data, map, code)
}
