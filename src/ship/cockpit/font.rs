//! A pixel font of three columns and five rows for the numbers and the units of the text
//! displays. The displays are small pictures painted on the CPU, so the font is a table.

/// Columns of a glyph.
pub const WIDTH: usize = 3;

/// Rows of a glyph.
pub const HEIGHT: usize = 5;

/// The glyphs: each row holds its three pixels in the low bits, the left one the highest.
const GLYPHS: [(char, [u8; HEIGHT]); 20] = [
    ('0', [7, 5, 5, 5, 7]),
    ('1', [2, 6, 2, 2, 7]),
    ('2', [7, 1, 7, 4, 7]),
    ('3', [7, 1, 7, 1, 7]),
    ('4', [5, 5, 7, 1, 1]),
    ('5', [7, 4, 7, 1, 7]),
    ('6', [7, 4, 7, 5, 7]),
    ('7', [7, 1, 1, 2, 2]),
    ('8', [7, 5, 7, 5, 7]),
    ('9', [7, 5, 7, 1, 7]),
    ('.', [0, 0, 0, 0, 2]),
    ('-', [0, 0, 7, 0, 0]),
    ('/', [1, 1, 2, 4, 4]),
    ('m', [0, 7, 7, 5, 5]),
    ('k', [5, 5, 6, 5, 5]),
    ('M', [5, 7, 7, 5, 5]),
    ('g', [0, 7, 5, 7, 1]),
    ('l', [6, 2, 2, 2, 7]),
    ('y', [5, 5, 7, 1, 6]),
    ('s', [3, 4, 2, 1, 6]),
];

/// Returns the rows of the glyph of `c`: a space and any character the font does not know are
/// blank.
pub fn glyph(c: char) -> [u8; HEIGHT] {
    GLYPHS
        .iter()
        .find(|(known, _)| *known == c)
        .map_or([0; HEIGHT], |(_, rows)| *rows)
}
