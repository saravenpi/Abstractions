//! One-shot sounds: a fixed pool of blip voices fed by cues, each heard through its own ears
//! from where its cue starts.

pub(super) mod calls;
mod chirps;
mod hits;
pub(super) mod sounds;
pub(super) mod voice;

use voice::{Blip, Voice};

use super::cue::AudioCue;
use super::dsp::rand::Rand;
use super::spatial::Spot;

/// Blips that can sound at once. A cue that finds no free voice loses the blips that do not fit.
const VOICES: usize = 16;

/// Voices a call of an animal leaves free for every other cue.
const CALL_SPARE: usize = 6;

/// Voices a blow leaves free for every other cue: with 16 voices, blows hold 10 at most, about three
/// of them at once.
const HIT_SPARE: usize = 6;

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
    /// What the last frame sent the cabin around ears in the seat.
    sent: f32,
}

impl Pool {
    /// Builds a silent pool for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            voices: std::array::from_fn(|i| Voice::new(0x9E37_79B9_u32.wrapping_mul(i as u32 + 1))),
            rand: Rand::new(VARIATION_SEED, 0),
            rate,
            sent: 0.0,
        }
    }

    /// Starts every blip of `cue` on free voices, heard as `spot` says. A cue under the floor of
    /// hearing takes no voice. An animal call or a blow only starts when it leaves voices free for
    /// the rest.
    pub fn play(&mut self, cue: AudioCue, spot: &Spot) {
        if !spot.heard() {
            return;
        }
        match cue {
            AudioCue::Call(call) => {
                let (blips, count) = calls::call(&call, &mut self.rand);
                self.spare(&blips[..count], CALL_SPARE, spot);
            }
            AudioCue::Hit(sound) => {
                let (blips, count) = hits::hit(&sound, &mut self.rand);
                self.spare(&blips[..count], HIT_SPARE, spot);
            }
            _ => {
                let (blips, pitch, gain) = sounds::recipe(cue, &mut self.rand);
                self.start(blips, (pitch, gain), spot);
            }
        }
    }

    /// Starts `blips` from `spot` only when `spare` voices stay free for the rest.
    fn spare(&mut self, blips: &[Blip], spare: usize, spot: &Spot) {
        let free = self.voices.iter().filter(|voice| voice.idle()).count();
        if free >= blips.len() + spare {
            self.start(blips, (1.0, 1.0), spot);
        }
    }

    /// Starts `blips` on free voices at a pitch factor and a level, late by the delay of `spot`
    /// and heard from it.
    fn start(&mut self, blips: &[Blip], (pitch, gain): (f32, f32), spot: &Spot) {
        let rate = self.rate;
        let free = self.voices.iter_mut().filter(|voice| voice.idle());
        for (voice, blip) in free.zip(blips) {
            let late = Blip {
                delay: blip.delay + spot.delay,
                ..*blip
            };
            voice.start(late, pitch, gain, rate);
            voice.ear.place(spot, rate);
        }
    }

    /// Returns how many voices are free.
    #[cfg(test)]
    pub fn free(&self) -> usize {
        self.voices.iter().filter(|voice| voice.idle()).count()
    }

    /// Returns true while any blip is sounding or waiting to.
    #[cfg(test)]
    pub fn busy(&self) -> bool {
        self.voices.iter().any(|voice| !voice.idle())
    }

    /// Returns what the last frame sent the cabin, for its reflections.
    pub fn sent(&self) -> f32 {
        self.sent
    }

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        let (mut sum, mut sent) = ([0.0; 2], 0.0);
        for voice in self.voices.iter_mut().filter(|voice| !voice.idle()) {
            let sample = voice.frame();
            let [left, right] = voice.ear.frame(sample, sample);
            sum = [sum[0] + left, sum[1] + right];
            sent += 0.5 * (left + right) * voice.ear.room();
        }
        self.sent = sent;
        sum
    }
}
