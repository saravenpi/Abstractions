//! One-shot sounds: a fixed pool of blip voices fed by cues.

pub(super) mod sounds;
pub(super) mod voice;

use voice::Voice;

use super::cue::AudioCue;
use super::dsp::rand::Rand;

/// Blips that can sound at once. A cue that finds no free voice loses the blips that do not fit.
const VOICES: usize = 16;

/// Seed of the small variations between two plays of the same cue.
const VARIATION_SEED: u64 = 0x5EED_C0E5;

/// The pool of blip voices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pool {
    /// The voices.
    voices: [Voice; VOICES],
    /// Source of pitch and level variations.
    rand: Rand,
    /// Sample rate in hertz.
    rate: f32,
}

impl Pool {
    /// Builds a silent pool for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            voices: std::array::from_fn(|i| Voice::new(0x9E37_79B9_u32.wrapping_mul(i as u32 + 1))),
            rand: Rand::new(VARIATION_SEED, 0),
            rate,
        }
    }

    /// Starts every blip of `cue` on free voices.
    pub fn play(&mut self, cue: AudioCue) {
        let (blips, pitch, gain) = sounds::recipe(cue, &mut self.rand);
        let free = self.voices.iter_mut().filter(|voice| voice.idle());
        for (voice, blip) in free.zip(blips) {
            voice.start(*blip, pitch, gain, self.rate);
        }
    }

    /// Returns true while any blip is sounding or waiting to.
    #[cfg(test)]
    pub fn busy(&self) -> bool {
        self.voices.iter().any(|voice| !voice.idle())
    }

    /// Renders one sample.
    pub fn frame(&mut self) -> f32 {
        self.voices.iter_mut().map(Voice::frame).sum()
    }
}
