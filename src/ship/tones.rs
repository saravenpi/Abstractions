//! The tones a triangle of a ship mesh can be painted in: a colour multiplier over the colour of
//! the material, so one material shows plating in several tones.

/// Linear colour multipliers of the tones over the bone of the hull: plain, the slate of the
/// second plates, the grey of the undersides, the charcoal of the frame and of the heat shield,
/// the black of bays and ports, and the accent colour.
const TONES: [[f32; 4]; 6] = [
    [1.0, 1.0, 1.0, 1.0],
    [0.26, 0.38, 0.57, 1.0],
    [0.16, 0.2, 0.27, 1.0],
    [0.045, 0.048, 0.058, 1.0],
    [0.012, 0.012, 0.015, 1.0],
    [1.2, 0.27, 0.03, 1.0],
];

/// The last tone of the palette.
pub const LAST: u8 = TONES.len() as u8 - 1;

/// Returns the colour multiplier of `tone`, in linear space.
pub fn paint(tone: u8) -> [f32; 4] {
    TONES[usize::from(tone).min(TONES.len() - 1)]
}
