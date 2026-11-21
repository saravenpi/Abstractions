//! Footsteps: a small pool of voices, one per footfall being heard. Every step is synthesised
//! from the ground under the foot and the force of the landing, and turned a little its own
//! way, so no two steps are the same sound.

mod material;
mod voice;

use voice::{Tread, Turn};

use super::cue::AudioCue;
use super::dsp::rand::Rand;
use super::shore::stereo;
use super::step::Step;

/// Footfalls that can sound at once. One more takes the voice nearest its end.
const VOICES: usize = 8;

/// How far the pitch of a step strays from its ground, either way, and the quietest a step is
/// against the loudest of its force.
const STRAY: (f32, f32) = (0.06, 0.8);

/// How far apart the two feet of a body are tuned, and how far to each side they stand.
const FEET: (f32, f32) = (0.03, 0.06);

/// The pool of footfall voices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Treads {
    /// The voices.
    voices: [Tread; VOICES],
    /// Source of the turn of each step.
    rand: Rand,
    /// Sample rate in hertz.
    rate: f32,
}

impl Treads {
    /// Builds a silent pool for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            voices: std::array::from_fn(|i| Tread::new(0x7EAD_5EED ^ (i as u32 + 1) * 0x9E37, rate)),
            rand: Rand::new(0x0F00_7FA1, 0),
            rate,
        }
    }

    /// Starts `step` on a free voice, or on the one nearest its end.
    pub fn play(&mut self, step: &Step) {
        let foot = if step.left { -1.0 } else { 1.0 };
        let turn = Turn {
            pitch: self.rand.range(1.0 - STRAY.0, 1.0 + STRAY.0) * (1.0 + FEET.0 * foot),
            level: self.rand.range(STRAY.1, 1.0),
            sides: stereo(step.side + FEET.1 * foot, step.near),
        };
        let rate = self.rate;
        if let Some(voice) = self.voices.iter_mut().min_by_key(|voice| voice.left()) {
            voice.start(step, &turn, rate);
        }
    }

    /// Plays `cue` if it is a footfall.
    pub fn hear(&mut self, cue: AudioCue) {
        if let AudioCue::Footstep(step) = cue {
            self.play(&step);
        }
    }

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        self.voices.iter_mut().fold([0.0; 2], |sum, voice| {
            let [left, right] = voice.frame();
            [sum[0] + left, sum[1] + right]
        })
    }
}
