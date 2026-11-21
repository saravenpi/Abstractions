//! One-shot sounds: a fixed pool of blip voices fed by cues, and the voices of the animals, each
//! heard through its own ears from where its cue starts.

mod chirps;
mod hits;
pub(super) mod sounds;
pub(super) mod voice;

use voice::{Blip, Voice};

use super::cue::AudioCue;
use super::dsp::rand::Rand;
use super::spatial::Spot;
use super::throat::Throats;
use super::throat::tune::BUDDY_UNDER;

/// Blips that can sound at once. A cue that finds no free voice loses the blips that do not fit.
const VOICES: usize = 16;

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
    /// The voices of the animals.
    throats: Throats,
}

impl Pool {
    /// Builds a silent pool for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            voices: std::array::from_fn(|i| Voice::new(0x9E37_79B9_u32.wrapping_mul(i as u32 + 1))),
            rand: Rand::new(VARIATION_SEED, 0),
            rate,
            sent: 0.0,
            throats: Throats::new(rate),
        }
    }

    /// Starts `cue` heard as `spot` says: an animal call on the voices of the animals, a blow only
    /// when it leaves voices free for the rest, anything else on free blip voices. A cue under the
    /// floor of hearing takes no voice. The companion speaks under a calling animal.
    pub fn play(&mut self, cue: AudioCue, spot: &Spot) {
        if !spot.heard() {
            return;
        }
        match cue {
            AudioCue::Call(call) => self.throats.play(call, spot),
            AudioCue::Hit(sound) => {
                let (blips, count) = hits::hit(&sound, &mut self.rand);
                let free = self.voices.iter().filter(|voice| voice.idle()).count();
                if free >= count + HIT_SPARE {
                    self.start(&blips[..count], (1.0, 1.0), spot);
                }
            }
            _ => {
                let (blips, pitch, gain) = sounds::recipe(cue, &mut self.rand);
                let busy = matches!(cue, AudioCue::Chirp(_)) && self.throats.busy();
                let under = if busy { BUDDY_UNDER } else { 1.0 };
                self.start(blips, (pitch, gain * under), spot);
            }
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

    /// Returns how many blip voices are free.
    #[cfg(test)]
    pub fn free(&self) -> usize {
        self.voices.iter().filter(|voice| voice.idle()).count()
    }

    /// Returns true while any blip or animal is sounding or waiting to.
    #[cfg(test)]
    pub fn busy(&self) -> bool {
        self.voices.iter().any(|voice| !voice.idle()) || self.throats.busy()
    }

    /// Returns the voices of the animals.
    #[cfg(test)]
    pub fn throats(&self) -> &Throats {
        &self.throats
    }

    /// Returns what the last frame sent the cabin, for its reflections.
    pub fn sent(&self) -> f32 {
        self.sent
    }

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        let (mut sum, mut sent) = (self.throats.frame(), 0.0);
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
