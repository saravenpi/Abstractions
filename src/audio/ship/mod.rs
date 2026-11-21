//! The ships: every ship in earshot is a voice built from its real state, and the whoosh of
//! crossing clouds rides with them.
//!
//! A voice layers what a real craft of this kind makes: the jet noise of the main engines and
//! of the lift nozzles, the whine of the pumps, the wind and the buffet on the airframe, the
//! warp drive, the mechanisms and the cabin bed. `hull` mixes them for a listener in the seat
//! or outside, `path` carries them through the air, `law` holds the physics and `tune` the
//! constants. The audio thread never allocates and never locks here: a voice costs about 700
//! floating point operations per sample while its ship is in earshot, and nothing otherwise.

mod air_tune;
mod airframe;
#[cfg(test)]
mod airframe_tests;
mod cabin;
mod drive;
pub(super) mod feed;
#[cfg(test)]
mod flow_tests;
pub(super) mod hull;
mod jet;
pub(super) mod law;
mod motor;
pub(super) mod path;
pub(super) mod pump;
pub(super) mod ring;
pub(super) mod tune;
mod whirl;
mod works;

use feed::{Deck, HULLS};
use hull::Hull;
use tune::{BUS_CEILING, WHOOSH_LEVEL};

use super::cue::AudioCue;
use super::dsp::filter::Svf;
use super::dsp::rand::Noise;
use super::spatial::Absorb;

/// Every ship voice and the whoosh of the clouds.
#[derive(Debug, Clone)]
pub struct Fleet {
    /// One voice per ship that can sound. The first is the ship this machine flies.
    hulls: [Hull; HULLS],
    /// Colour of the whoosh.
    whoosh: Svf,
    /// Noise source of the whoosh.
    noise: Noise,
    /// How deep the listener is in the cloud layer, as last tuned.
    cloud: f32,
    /// Sample rate in hertz.
    rate: f32,
}

/// Returns `x` bent under [`BUS_CEILING`]: nearly untouched when quiet, never above it.
fn press(x: f32) -> f32 {
    BUS_CEILING * (x / BUS_CEILING).tanh()
}

impl Fleet {
    /// Builds silent voices for a sample rate, tuned every `dt` seconds.
    pub fn new(rate: f32, dt: f32) -> Self {
        Self {
            hulls: std::array::from_fn(|slot| Hull::new(rate, slot as u32 + 1, dt)),
            whoosh: Svf::default(),
            noise: Noise::new(0x51A7_E0B3),
            cloud: 0.0,
            rate,
        }
    }

    /// Reads what the game says of every ship and retunes the voices, `dt` seconds after the
    /// last call, for a listener `cloud` deep in the cloud layer whose air takes the high end as
    /// `absorb` says. Call it a few hundred times per second.
    pub fn tune(&mut self, deck: &Deck, (cloud, absorb): (f32, Absorb), dt: f32) {
        for (slot, hull) in self.hulls.iter_mut().enumerate() {
            hull.tune(deck.read(slot), &absorb, dt);
        }
        self.whoosh.tune(800.0 + 1700.0 * cloud, 0.6, self.rate);
        self.cloud = cloud;
    }

    /// Lets the ship this machine flies answer a cue: its gear takes a landing.
    pub fn hear(&mut self, cue: AudioCue) {
        if let AudioCue::Land(strength) = cue {
            self.hulls[0].works.land(strength);
        }
    }

    /// Renders one stereo frame, for a listener as deep in the cloud layer of a planet as the
    /// last tuning said.
    pub fn frame(&mut self) -> [f32; 2] {
        let whoosh = self.whoosh.band(self.noise.tick()) * self.cloud * WHOOSH_LEVEL;
        let mut sum = [0.0; 2];
        for hull in &mut self.hulls {
            let [left, right] = hull.frame();
            sum = [sum[0] + left, sum[1] + right];
        }
        sum.map(|side| press(side) + whoosh)
    }
}
