//! One-shot sounds: a fixed pool of blip voices fed by cues.

pub(super) mod calls;
mod chirps;
mod hits;
pub(super) mod sounds;
mod steps;
pub(super) mod voice;

use voice::Voice;

use super::call::Call;
use super::cue::AudioCue;
use super::dsp::rand::Rand;
use super::hit::HitSound;
use super::shore::stereo;

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
    /// Left and right gains of what the companion says, from where it floats.
    buddy: [f32; 2],
}

impl Pool {
    /// Builds a silent pool for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            voices: std::array::from_fn(|i| Voice::new(0x9E37_79B9_u32.wrapping_mul(i as u32 + 1))),
            rand: Rand::new(VARIATION_SEED, 0),
            rate,
            buddy: [1.0; 2],
        }
    }

    /// Places the voice of the companion: on `side` from -1 (left) to 1 (right), `near` from 0
    /// (out of earshot) to 1 (beside the head). A chirp keeps the place it started at.
    pub fn locate(&mut self, side: f32, near: f32) {
        self.buddy = stereo(side, near);
    }

    /// Starts every blip of `cue` on free voices.
    pub fn play(&mut self, cue: AudioCue) {
        match cue {
            AudioCue::Call(call) => return self.call(call),
            AudioCue::Hit(sound) => return self.hit(sound),
            _ => {}
        }
        let (blips, pitch, gain) = sounds::recipe(cue, &mut self.rand);
        let free = self.voices.iter_mut().filter(|voice| voice.idle());
        let sides = match cue {
            AudioCue::Chirp(_) => self.buddy,
            _ => [1.0; 2],
        };
        for (voice, blip) in free.zip(blips) {
            voice.start(*blip, pitch, gain, self.rate);
            voice.sides = sides;
        }
    }

    /// Starts the blips of the animal `call` from where it is, only when [`CALL_SPARE`] voices stay free
    /// for the rest.
    fn call(&mut self, call: Call) {
        let (blips, count) = calls::call(&call, &mut self.rand);
        let free = self.voices.iter().filter(|voice| voice.idle()).count();
        if free < count + CALL_SPARE {
            return;
        }
        let sides = stereo(call.side, call.near);
        let idle = self.voices.iter_mut().filter(|voice| voice.idle());
        for (voice, blip) in idle.zip(&blips[..count]) {
            voice.start(*blip, 1.0, 1.0, self.rate);
            voice.sides = sides;
        }
    }

    /// Starts the blips of a blow or a swing from where it is, only when [`HIT_SPARE`] voices stay free
    /// for the rest: at most three blips for a blow.
    fn hit(&mut self, sound: HitSound) {
        let (blips, count) = hits::hit(&sound, &mut self.rand);
        let free = self.voices.iter().filter(|voice| voice.idle()).count();
        if free < count + HIT_SPARE {
            return;
        }
        let sides = stereo(sound.side, sound.near);
        let idle = self.voices.iter_mut().filter(|voice| voice.idle());
        for (voice, blip) in idle.zip(&blips[..count]) {
            voice.start(*blip, 1.0, 1.0, self.rate);
            voice.sides = sides;
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

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        self.voices.iter_mut().fold([0.0; 2], |sum, voice| {
            let sample = voice.frame();
            [
                sum[0] + sample * voice.sides[0],
                sum[1] + sample * voice.sides[1],
            ]
        })
    }
}
