//! One-shot sounds: a fixed pool of blip voices fed by cues, and the voices of the animals.

mod chirps;
mod hits;
pub(super) mod sounds;
mod steps;
pub(super) mod voice;

use voice::Voice;

use super::cue::AudioCue;
use super::dsp::rand::Rand;
use super::hit::HitSound;
use super::shore::stereo;
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
    /// Left and right gains of what the companion says, from where it floats.
    buddy: [f32; 2],
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
            buddy: [1.0; 2],
            throats: Throats::new(rate),
        }
    }

    /// Places the voice of the companion: on `side` from -1 (left) to 1 (right), `near` from 0
    /// (out of earshot) to 1 (beside the head). A chirp keeps the place it started at.
    pub fn locate(&mut self, side: f32, near: f32) {
        self.buddy = stereo(side, near);
    }

    /// Starts every blip of `cue` on free voices. The companion speaks under a calling animal.
    pub fn play(&mut self, cue: AudioCue) {
        match cue {
            AudioCue::Call(call) => return self.throats.play(call),
            AudioCue::Hit(sound) => return self.hit(sound),
            _ => {}
        }
        let (blips, pitch, gain) = sounds::recipe(cue, &mut self.rand);
        let free = self.voices.iter_mut().filter(|voice| voice.idle());
        let under = if self.throats.busy() {
            BUDDY_UNDER
        } else {
            1.0
        };
        let sides = match cue {
            AudioCue::Chirp(_) => self.buddy.map(|side| side * under),
            _ => [1.0; 2],
        };
        for (voice, blip) in free.zip(blips) {
            voice.start(*blip, pitch, gain, self.rate);
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

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        let animals = self.throats.frame();
        self.voices.iter_mut().fold(animals, |sum, voice| {
            let sample = voice.frame();
            [
                sum[0] + sample * voice.sides[0],
                sum[1] + sample * voice.sides[1],
            ]
        })
    }
}
