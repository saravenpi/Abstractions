//! The plating of the hull: which tone each panel of a band of the fuselage takes. The back and
//! the upper flanks are the bone plates, the flanks under the chine are the cool grey, and the
//! belly is the charcoal of the heat shield. The cap of the nose is charcoal all round.

/// Tones of the eight panels of a band of the fuselage, from the right flank round the back, the
/// left flank and the belly.
pub fn plating() -> [u8; 8] {
    [0, 0, 0, 0, 1, 3, 3, 1]
}

/// Tones of the eight panels of the band `band` of the nose, counted from the tip: the first is
/// the charcoal cap, which takes the heat of an entry, and the rest is plated like the hull.
pub fn cone(band: usize) -> [u8; 8] {
    match band {
        0 => [3; 8],
        _ => plating(),
    }
}
