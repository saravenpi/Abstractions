//! The plating of the hull: which tone each panel of a band of the fuselage takes. The upper
//! plates alternate between two tones along the length, the flanks and the belly are darker,
//! and the belly is the charcoal of the heat shield. The nose cone is the orange trim.

/// Tones of the eight panels of a band of the plain fuselage, from the right flank round the
/// roof, the left flank and the belly. `band` counts the bands from the nose.
pub fn plating(band: usize) -> [u8; 8] {
    let (roof, shoulder) = if band.is_multiple_of(2) {
        (0, 1)
    } else {
        (1, 0)
    };
    [shoulder, roof, roof, shoulder, 2, 3, 3, 2]
}

/// Tones of the eight panels of a band of the nose cone: orange, over the dark heat shield.
pub fn cone(_band: usize) -> [u8; 8] {
    [5, 5, 5, 5, 5, 3, 3, 5]
}
