//! Footsteps: a small pool of voices, one per footfall being heard. Every step is synthesised
//! from the ground under the foot, the speed and the mass it strikes with and the weight it
//! carries, turned a little its own way, and carried to the listener as the sound of a ship
//! is: late by its distance over the speed of sound, spread as `1 / r`, dulled by the air.
//!
//! The model follows the footstep chapter of Farnell, "Designing Sound" (MIT Press, 2010): a
//! step is a heel strike, a roll and a toe, each exciting a model of the ground, and Turchet,
//! "Footstep sounds synthesis: design, implementation, and evaluation of foot-floor interactions,
//! surface materials, shoe types, and walkers' features", Applied Acoustics 107, 2016:
//! https://doi.org/10.1016/j.apacoust.2015.05.013

mod grain;
mod material;
mod sweep;
pub(super) mod tune;
mod voice;

use tune::{
    BODY, EAR, EARSHOT, FEET, FORCE_REF, GROWTH, LOAD_REF, MASS_REF, ROLL_STRAY, SOUND, STRAY,
    TICK, WADE,
};
use voice::{Shape, Tread};

use super::cue::{AudioCue, StepSurface};
use super::dsp::filter::pole;
use super::dsp::rand::Rand;
use super::ship::law::absorb_hz;
use super::shore::stereo;
use super::step::{Contact, Step};

/// Footfalls that can sound at once. One more takes the voice nearest its end.
const VOICES: usize = 8;

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

/// Returns the level a step `far` metres away arrives at against the listener's own: 1 within
/// reach of the ears, then `1 / r`, and nothing at the edge of earshot.
pub fn near(far: f32) -> f32 {
    let edge = 1.0 - (far / EARSHOT).clamp(0.0, 1.0).powi(4);
    EAR / far.max(EAR) * edge
}

impl Treads {
    /// Builds a silent pool for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            voices: std::array::from_fn(|i| {
                Tread::new(0x7EAD_5EED ^ (i as u32 + 1).wrapping_mul(0x9E37_79B9), rate)
            }),
            rand: Rand::new(0x0F00_7FA1, 0),
            rate,
        }
    }

    /// Returns the gains of the air and of the body, the left and right gains, and the
    /// coefficient of the low-pass of the air, for `step` and its `foot`, -1 for the left.
    fn ears(&self, step: &Step, foot: f32) -> ((f32, f32), [f32; 2], f32) {
        let own = step.far <= 0.0;
        let air = step.air.clamp(0.0, 1.0);
        let (inner, side) = if own {
            (BODY, FEET.1 * foot)
        } else {
            (0.0, step.side)
        };
        let dull = pole(absorb_hz(step.far, air.max(0.02)), self.rate);
        ((air, inner), stereo(side, near(step.far)), dull)
    }

    /// Returns the samples until `step` arrives, until its toe lands and until it lifts: not
    /// before the wet sweep of the foot going in is over.
    fn times(&mut self, step: &Step) -> (u32, u32, u32) {
        let roll = step.roll * self.rand.range(1.0 - ROLL_STRAY, 1.0 + ROLL_STRAY);
        let toe = match step.contact {
            Contact::Stride => (roll * self.rate) as u32 + 1,
            _ => 0,
        };
        let late = step.lift.max(roll).max(material::of(step.surface).wet.2);
        let lift = (late * self.rate) as u32 + 1;
        ((step.far / SOUND * self.rate) as u32 + 1, toe, lift)
    }

    /// Works out how `step` sounds this time: its turn, its levels from the physics of the
    /// strike, and its way to the listener.
    fn shape(&mut self, step: &Step) -> Shape {
        let foot = if step.left { -1.0 } else { 1.0 };
        let pitch = self.rand.range(1.0 - STRAY.0, 1.0 + STRAY.0) * (1.0 + FEET.0 * foot);
        let level = self.rand.range(STRAY.1, 1.0) * (1.0 + FEET.2 * foot);
        let force = (step.force.max(0.0) / FORCE_REF).powf(GROWTH);
        let wade = match step.surface {
            StepSurface::Water => (step.depth / WADE.0).clamp(WADE.1, WADE.2),
            _ => 1.0,
        };
        let hard = ((step.force - TICK.0) / (1.0 - TICK.0)).clamp(0.0, 1.0);
        let (ears, sides, dull) = self.ears(step, foot);
        Shape {
            pitch: (pitch, pitch * (MASS_REF / step.mass.max(1.0)).sqrt()),
            push: force * level * wade,
            grains: (step.load.max(0.0) / LOAD_REF, force.sqrt() * level * wade),
            spread: self.rand.unit(),
            times: self.times(step),
            ears,
            sides,
            dull,
            tick: TICK.3 * hard * f32::from(u8::from(step.contact == Contact::Landing)),
        }
    }

    /// Starts `step` on a free voice, or on the one nearest its end. A step out of earshot,
    /// or someone else's in a vacuum, is not heard.
    pub fn play(&mut self, step: &Step) {
        let silent = step.far > 0.0 && step.air <= 0.0;
        if step.far >= EARSHOT || silent {
            return;
        }
        let shape = self.shape(step);
        if let Some(voice) = self.voices.iter_mut().min_by_key(|voice| voice.left()) {
            voice.start(step, shape);
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
