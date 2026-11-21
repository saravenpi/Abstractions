//! The voices of animals, built from their bodies: a source through a tract, then the air.
//!
//! Each call follows the source-filter model of animal voices. The source is the organ the body has:
//! vocal folds that pulse with jitter and shimmer and, when strained, subharmonics and chaos; a syrinx
//! that sweeps a pure tone; a file whose teeth ring a shell; or breath alone, a hiss or a snort. The
//! filter is the vocal tract, three formants spaced by its length, so a long body or a long neck sounds
//! big whatever its pitch (Fitch 1997, <https://doi.org/10.1121/1.421048>). The way to the ear takes the
//! highs the air absorbs with distance and, from under water, nearly everything the surface reflects.
//!
//! A pool of a few voices sounds the calls. Contact calls take turns: no more than [`tune::CHORUS`] at
//! once, and an animal does not start over another of its species that has just begun, so a dawn
//! chorus is a scene of answers and not a wall of noise. Alarms, threats and cries of pain never wait:
//! with no voice free they take the place of the quietest contact call.

mod folds;
mod notes;
mod plan;
mod pulses;
mod shape;
mod tract;
pub mod tune;
mod voice;

pub(super) use plan::{Plan, plan};
#[cfg(test)]
pub(super) use shape::Source;

use super::call::Call;
use super::dsp::rand::Rand;
use super::shore::stereo;
use shape::urgent;
use tune::{CHORUS, THROATS, TURN};
use voice::Throat;

/// Seed of the small variations between two calls of the same animal.
const SEED: u64 = 0x7A0A_75E5;

/// The voices of the animals in earshot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Throats {
    /// The voices.
    voices: [Throat; THROATS],
    /// The voice print of the species each voice sounds, and whether its call is urgent.
    who: [(u8, bool); THROATS],
    /// Source of the variations.
    rand: Rand,
}

impl Throats {
    /// Builds a silent pool for a sample `rate`.
    pub fn new(rate: f32) -> Self {
        Self {
            voices: std::array::from_fn(|i| {
                Throat::new(
                    0x85EB_CA6B_u32.wrapping_mul(i as u32 + 1),
                    rate,
                    Plan::SILENT,
                )
            }),
            who: [(0, false); THROATS],
            rand: Rand::new(SEED, 0),
        }
    }

    /// Returns the quietest voice sounding a call that can wait, if any.
    fn steal(&self) -> Option<usize> {
        (0..THROATS)
            .filter(|i| !self.voices[*i].idle() && !self.who[*i].1)
            .min_by(|a, b| {
                self.voices[*a]
                    .plan
                    .gain
                    .total_cmp(&self.voices[*b].plan.gain)
            })
    }

    /// Returns the voice `call` may take, or nothing when it must let others finish.
    fn slot(&self, call: &Call) -> Option<usize> {
        let pressing = urgent(call.cry);
        let live = |i: &usize| !self.voices[*i].idle();
        if !pressing {
            let calm = (0..THROATS)
                .filter(live)
                .filter(|i| !self.who[*i].1)
                .count();
            let begun = |i: usize| {
                let v = &self.voices[i];
                self.who[i].0 == call.voice.print && v.at < TURN * v.plan.seconds()
            };
            if calm >= CHORUS || (0..THROATS).filter(live).any(begun) {
                return None;
            }
        }
        let free = self.voices.iter().position(Throat::idle);
        free.or_else(|| if pressing { self.steal() } else { None })
    }

    /// Starts `call` from where it is, if a voice can take it.
    pub fn play(&mut self, call: Call) {
        let Some(i) = self.slot(&call) else {
            return;
        };
        let plan = plan(&call, &mut self.rand);
        let seed = self.rand.unit();
        self.voices[i].start(plan, stereo(call.side, 1.0), seed);
        self.who[i] = (call.voice.print, urgent(call.cry));
    }

    /// Returns true while any animal calls.
    pub fn busy(&self) -> bool {
        self.voices.iter().any(|voice| !voice.idle())
    }

    /// Returns how many animals call now.
    #[cfg(test)]
    pub fn sounding(&self) -> usize {
        self.voices.iter().filter(|voice| !voice.idle()).count()
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
