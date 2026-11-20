//! The tones a triangle of a ship mesh can be painted in: a colour multiplier over the colour of
//! the material, so one material shows plating in several tones.

/// Linear colour multipliers of the tones: plain, a lighter plate, the grey of the underside,
/// the charcoal of the heat shield, the black of bays and ports, and the orange of the trim on
/// the grey of the hull.
const TONES: [[f32; 4]; 6] = [
    [1.0, 1.0, 1.0, 1.0],
    [0.88, 0.9, 0.93, 1.0],
    [0.62, 0.64, 0.68, 1.0],
    [0.22, 0.23, 0.26, 1.0],
    [0.1, 0.1, 0.12, 1.0],
    [1.18, 0.56, 0.15, 1.0],
];

/// The last tone of the palette.
pub const LAST: u8 = TONES.len() as u8 - 1;

/// Returns the colour multiplier of `tone`, in linear space.
pub fn paint(tone: u8) -> [f32; 4] {
    TONES[usize::from(tone).min(TONES.len() - 1)]
}
