//! Oscillators: a phase counter and cheap sine and triangle shapes read from it.

/// Returns an approximate sine of `phase`, given in turns from 0 to 1.
///
/// A corrected parabola: the error stays under 0.1 percent, far below what a soft pad reveals.
pub fn sine(phase: f32) -> f32 {
    let t = 2.0 * phase - 1.0;
    let y = 4.0 * t * (1.0 - t.abs());
    -(0.225 * (y * y.abs() - y) + y)
}

/// Returns a triangle wave of `phase` in turns, in step with [`sine`].
pub fn triangle(phase: f32) -> f32 {
    1.0 - 4.0 * ((phase + 0.25).fract() - 0.5).abs()
}

/// A phase counter in turns, always inside 0 to 1.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Phasor(pub f32);

impl Phasor {
    /// Returns the current phase, then moves it forward by `step` turns.
    pub fn tick(&mut self, step: f32) -> f32 {
        let now = self.0;
        let next = now + step;
        self.0 = next - next.floor();
        now
    }
}

/// Returns the frequency in hertz of a MIDI note number, 69 being 440 Hz.
pub fn note_hz(note: f32) -> f32 {
    440.0 * ((note - 69.0) / 12.0).exp2()
}
