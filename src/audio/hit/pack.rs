//! Packing a [`HitSound`] into 16 bits, so it can cross to the audio thread as one word.

use super::sound::{HitSound, Kind};

/// Lowest frequency in hertz a packed sound carries; steps go up from it in eighths of an octave.
const BASE_HZ: f32 = 40.0;

/// Pitch steps per octave.
const STEPS_PER_OCTAVE: f32 = 8.0;

/// Levels of the loudness, and of the brightness.
const LEVELS: [f32; 2] = [15.0, 7.0];

impl HitSound {
    /// Packs the sound: kind 2 bits, loudness 4, pitch 6, brightness 3, wet 1. The pitch is rounded to
    /// an eighth of an octave and the rest to their levels.
    pub fn pack(self) -> u32 {
        let step = (STEPS_PER_OCTAVE * (self.hz / BASE_HZ).max(1.0).log2()).round() as u32;
        let level = |x: f32, n: f32| (x.clamp(0.0, 1.0) * n).round() as u32;
        (self.kind as u32)
            | level(self.level, LEVELS[0]) << 2
            | step.min(63) << 6
            | level(self.hard, LEVELS[1]) << 12
            | u32::from(self.wet) << 15
    }

    /// Unpacks a sound packed by [`HitSound::pack`], or nothing when the bits hold none.
    pub fn unpack(bits: u32) -> Option<Self> {
        let field = |shift: u32, width: u32| ((bits >> shift) & ((1 << width) - 1)) as f32;
        let kind = match field(0, 2) as u32 {
            0 => Kind::Clean,
            1 => Kind::Tick,
            2 => Kind::Whoosh,
            _ => return None,
        };
        Some(Self {
            kind,
            level: field(2, 4) / LEVELS[0],
            hz: BASE_HZ * (field(6, 6) / STEPS_PER_OCTAVE).exp2(),
            hard: field(12, 3) / LEVELS[1],
            wet: field(15, 1) == 1.0,
        })
    }
}
