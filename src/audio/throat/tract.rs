//! The tract of a calling animal: three formants of the vocal tract, then, for an animal that calls
//! from under water, the surface it crosses. The air on the way is the spatial path's.

use super::tune::MUFFLE;
use crate::audio::dsp::filter::Svf;

/// Resonance of the first, second and third formant.
const FORMANT_Q: [f32; 3] = [4.0, 6.0, 8.0];

/// Level of the first, second and third formant.
const FORMANT_GAIN: [f32; 3] = [1.0, 0.6, 0.35];

/// Share of the source that passes the formants untouched: the body of the sound.
const DIRECT: f32 = 0.15;

/// Resonance of the low-pass of the water: no peak.
const FLAT_Q: f32 = 0.707;

/// The tract of one calling animal, and the surface of the water over one that calls from under it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Tract {
    /// The three formants.
    formants: [Svf; 3],
    /// The low-pass of the surface of the water, when the call crosses it.
    water: Option<Svf>,
    /// True when the source passes the formants untouched: a syrinx tunes its tract to its note.
    open: bool,
}

impl Tract {
    /// Tunes the tract to `formants` in hertz, or opens it for a source that needs none, muffled
    /// by the surface of the water when `muffled`.
    pub fn tune(&mut self, (formants, open): ([f32; 3], bool), muffled: bool, rate: f32) {
        *self = Self {
            open,
            ..Self::default()
        };
        for ((filter, hz), q) in self.formants.iter_mut().zip(formants).zip(FORMANT_Q) {
            filter.tune(hz, q, rate);
        }
        self.water = muffled.then(|| {
            let mut surface = Svf::default();
            surface.tune(MUFFLE.0, FLAT_Q, rate);
            surface
        });
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

    /// Runs one sample of the shaped voice through the surface of the water, if it crosses one.
    pub fn carry(&mut self, x: f32) -> f32 {
        match &mut self.water {
            Some(surface) => surface.low(x),
            None => x,
        }
    }
}
