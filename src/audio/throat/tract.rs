//! The tract and the way to the ear: three formants of the vocal tract, then the highs the air takes
//! on the way and, from under water, the surface.

use super::tune::{AIR, AIR_CORNER_DB, AIR_TOP, MUFFLE};
use crate::audio::dsp::filter::Svf;

/// Resonance of the first, second and third formant.
const FORMANT_Q: [f32; 3] = [4.0, 6.0, 8.0];

/// Level of the first, second and third formant.
const FORMANT_GAIN: [f32; 3] = [1.0, 0.6, 0.35];

/// Share of the source that passes the formants untouched: the body of the sound.
const DIRECT: f32 = 0.15;

/// Resonance of the low-passes of the air and of the water: no peak.
const FLAT_Q: f32 = 0.707;

/// Returns the corner in hertz of the low-pass that stands for `metres` of air: where the air takes
/// [`AIR_CORNER_DB`] decibels, from an absorption that grows as a power of the frequency.
pub fn air(metres: f32) -> f32 {
    let (at_khz, power) = AIR;
    let khz = (AIR_CORNER_DB / (at_khz * metres.max(1.0))).powf(1.0 / power);
    (1000.0 * khz).min(AIR_TOP)
}

/// The tract of one calling animal and the way from it to the ear.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Tract {
    /// The three formants.
    formants: [Svf; 3],
    /// Two low-passes in a row: the air, and the air again or the water.
    way: [Svf; 2],
    /// True when the source passes the formants untouched: a syrinx tunes its tract to its note.
    open: bool,
}

impl Tract {
    /// Tunes the tract to `formants` in hertz, or opens it for a source that needs none, and the way
    /// to a low-pass at `air` hertz, muffled by the water when `muffled`.
    pub fn tune(
        &mut self,
        (formants, open): ([f32; 3], bool),
        (air, muffled): (f32, bool),
        rate: f32,
    ) {
        *self = Self {
            open,
            ..Self::default()
        };
        for ((filter, hz), q) in self.formants.iter_mut().zip(formants).zip(FORMANT_Q) {
            filter.tune(hz, q, rate);
        }
        let water = if muffled { air.min(MUFFLE.0) } else { air };
        self.way[0].tune(air, FLAT_Q, rate);
        self.way[1].tune(water, FLAT_Q, rate);
    }

    /// Runs one sample of the source through the formants.
    pub fn shape(&mut self, x: f32) -> f32 {
        if self.open {
            return x;
        }
        let rung = self
            .formants
            .iter_mut()
            .zip(FORMANT_GAIN)
            .fold(0.0, |sum, (filter, gain)| sum + gain * filter.band(x));
        rung + DIRECT * x
    }

    /// Runs one sample of the shaped voice along the way to the ear.
    pub fn carry(&mut self, x: f32) -> f32 {
        let [first, second] = &mut self.way;
        second.low(first.low(x))
    }
}
