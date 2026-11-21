//! The noise of one kind of jet: the broad roar of the turbulent mixing, the hiss of the fine
//! turbulence near the nozzle, one more band for the shock cells or for the ground, the crackle
//! of a hard burn in air, and the rumble the engine sends through the airframe.
//!
//! Nothing here is a tone. The roar swells and sinks with two slow random walks, as the eddies
//! of a real plume do, so it never sounds like noise through a fixed filter.

use super::law::{band_gain, loudness, peak_hz};
use super::tune::{CRACKLE, CRACKLE_SHAPE, EXTRA_Q, FINE, MIX_LEVEL, MIX_Q, RUMBLE, SURGE};
use crate::audio::dsp::filter::{Lag, Svf, pole};
use crate::audio::dsp::rand::Noise;
use crate::render::smoothstep;

/// What a jet follows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JetCtl {
    /// Thrust level: 1 at full throttle.
    pub level: f32,
    /// Density of the air as a share of the home sea level.
    pub air: f32,
    /// Speed of sound in the air, in metres per second.
    pub sound: f32,
    /// Share of the mixing noise that goes toward the listener.
    pub lobe: f32,
    /// Centre in hertz of the extra band: the shock cells, or the plume on the ground.
    pub extra_hz: f32,
    /// Level of the extra band at full thrust.
    pub extra: f32,
}

/// The gains of the layers of a jet, set a few hundred times per second.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Gains {
    /// Mixing noise.
    mix: f32,
    /// Fine turbulence.
    fine: f32,
    /// Extra band.
    extra: f32,
    /// Peak of a crackle shock.
    spike: f32,
    /// Noise value above which a sample carries a shock.
    floor: f32,
    /// Rumble through the airframe.
    rumble: f32,
}

/// Returns the coefficient of a random walk at `hz` and the gain that brings it to a level of 1.
pub(super) fn swell(hz: f32, rate: f32) -> (f32, f32) {
    let k = pole(hz, rate);
    (k, (3.0 * (2.0 - k) / k).sqrt())
}

/// One kind of jet of a ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jet {
    /// Colours: the mixing noise, the fine turbulence, the extra band and the rumble.
    bands: [Svf; 4],
    /// Gains of the layers.
    gains: Gains,
    /// The fast and the slow random walk of the roar.
    surge: [Lag; 2],
    /// Coefficient and gain of each random walk.
    swell: [(f32, f32); 2],
    /// The decay of the crackle shocks.
    crack: Lag,
    /// Coefficient of that decay.
    snap: f32,
    /// Noise source.
    noise: Noise,
    /// Exit diameter of one nozzle in metres.
    diameter: f32,
    /// Sample rate in hertz.
    rate: f32,
}

impl Jet {
    /// Builds a silent jet with nozzles of `diameter` metres, with its own noise `seed`.
    pub fn new(rate: f32, seed: u32, diameter: f32) -> Self {
        let mut bands = [Svf::default(); 4];
        bands[3].tune(RUMBLE.0, RUMBLE.1, rate);
        Self {
            bands,
            gains: Gains {
                floor: 1.0,
                ..Gains::default()
            },
            surge: [Lag::default(); 2],
            swell: SURGE.map(|(hz, _)| swell(hz, rate)),
            crack: Lag::default(),
            snap: pole(CRACKLE_SHAPE.2, rate),
            noise: Noise::new(seed),
            diameter,
            rate,
        }
    }

    /// Retunes the bands and sets the gains. Call it a few hundred times per second.
    pub fn tune(&mut self, ctl: &JetCtl) {
        let peak = peak_hz(ctl.sound, self.diameter, ctl.level);
        let fine = (peak * FINE.0).min(FINE.1);
        let loud = loudness(ctl.level);
        let hot = smoothstep(CRACKLE.0, CRACKLE.1, ctl.level) * ctl.air.min(1.0);
        self.bands[0].tune(peak, MIX_Q, self.rate);
        self.bands[1].tune(fine, FINE.2, self.rate);
        self.bands[2].tune(ctl.extra_hz, EXTRA_Q, self.rate);
        self.gains = Gains {
            mix: loud * ctl.lobe * MIX_LEVEL * band_gain(peak, MIX_Q, self.rate),
            fine: loud * (0.5 + 0.5 * ctl.lobe) * FINE.3 * band_gain(fine, FINE.2, self.rate),
            extra: loud * ctl.extra * band_gain(ctl.extra_hz, EXTRA_Q, self.rate),
            spike: hot * ctl.lobe * CRACKLE_SHAPE.1 / CRACKLE_SHAPE.0,
            floor: 1.0 - CRACKLE_SHAPE.0 * hot,
            rumble: loud * RUMBLE.2 * band_gain(RUMBLE.0, 1.0 / RUMBLE.1, self.rate),
        };
    }

    /// Renders one sample: what the jet radiates into the air at the reference distance, and
    /// what it sends through the airframe.
    pub fn frame(&mut self) -> (f32, f32) {
        let white = self.noise.tick();
        let [(fast, quick), (slow, lazy)] = self.swell;
        let wobble = self.surge[0].to(self.noise.tick(), fast) * quick * SURGE[0].1
            + self.surge[1].to(self.noise.tick(), slow) * lazy * SURGE[1].1;
        let surge = (1.0 + wobble).max(0.0);
        let gains = self.gains;
        let [mix, fine, extra, rumble] = &mut self.bands;
        let spike = (white - gains.floor).max(0.0) * gains.spike;
        let roar = mix.band(white) * gains.mix + extra.band(white) * gains.extra;
        let air = roar * surge + fine.band(white) * gains.fine + self.crack.to(spike, self.snap);
        (air, rumble.low(white) * gains.rumble * surge)
    }
}
