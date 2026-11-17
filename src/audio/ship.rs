//! The ship: engine hum, hull rumble and the whoosh of crossing clouds.

use super::dsp::filter::{Lag, Svf, pole};
use super::dsp::osc::{Phasor, sine};
use super::dsp::rand::Noise;

/// Level of the engine at full throttle.
pub const ENGINE_LEVEL: f32 = 0.1;

/// Level of the rumble at full dynamic pressure.
pub const RUMBLE_LEVEL: f32 = 1.4;

/// Level of the whoosh in the middle of a cloud.
pub const WHOOSH_LEVEL: f32 = 0.19;

/// Engine pitch in hertz at idle.
pub const IDLE_HZ: f32 = 41.0;

/// Engine pitch in hertz gained at full throttle.
pub const THROTTLE_HZ: f32 = 68.0;

/// Corner in hertz of the engine sound heard from the cabin in space.
const SPACE_HZ: f32 = 230.0;

/// What the ship sounds follow, each from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipCtl {
    /// Engine throttle.
    pub throttle: f32,
    /// Dynamic pressure on the hull.
    pub pressure: f32,
    /// Depth inside the cloud layer.
    pub cloud: f32,
    /// 1 inside the ship cabin. The engine is only heard from there.
    pub cabin: f32,
    /// 1 in space: the engine is muffled and nothing outside is heard.
    pub space: f32,
}

/// The ship voices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ship {
    /// Engine partials: fundamental, a beating twin, second and third harmonics.
    hum: [Phasor; 4],
    /// Engine hiss colour.
    hiss: Svf,
    /// Rumble colour.
    rumble: Svf,
    /// Whoosh colour.
    whoosh: Svf,
    /// Cabin wall: a low-pass on the engine.
    wall: Lag,
    /// Cabin wall coefficient.
    k: f32,
    /// Noise source.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Ship {
    /// Builds a silent ship for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            hum: [Phasor(0.0), Phasor(0.31), Phasor(0.62), Phasor(0.17)],
            hiss: Svf::default(),
            rumble: Svf::default(),
            whoosh: Svf::default(),
            wall: Lag::default(),
            k: 1.0,
            noise: Noise::new(0x51A7_E0B3),
            rate,
        }
    }

    /// Retunes the filters. Call it a few hundred times per second.
    pub fn tune(&mut self, ctl: &ShipCtl) {
        self.hiss
            .tune(350.0 + 1100.0 * ctl.throttle, 0.7, self.rate);
        self.rumble.tune(60.0 + 70.0 * ctl.pressure, 1.1, self.rate);
        self.whoosh.tune(800.0 + 1700.0 * ctl.cloud, 0.6, self.rate);
        let open = 500.0 + 3800.0 * ctl.throttle;
        self.k = pole(open * (SPACE_HZ / open).powf(ctl.space), self.rate);
    }

    /// Renders one sample.
    pub fn frame(&mut self, ctl: &ShipCtl) -> f32 {
        let step = (IDLE_HZ + THROTTLE_HZ * ctl.throttle) / self.rate;
        let bright = 0.25 + 0.65 * ctl.throttle;
        let [a, b, c, d] = &mut self.hum;
        let hum = sine(a.tick(step))
            + 0.6 * sine(b.tick(step * 1.011))
            + bright * 0.5 * sine(c.tick(step * 2.0))
            + bright * bright * 0.35 * sine(d.tick(step * 3.0));
        let raw = self.noise.tick();
        let hiss = self.hiss.band(raw) * ctl.throttle * 1.5;
        let power = ctl.cabin * (0.3 + 0.7 * ctl.throttle) * ENGINE_LEVEL;
        let engine = self.wall.to((hum + hiss) * power, self.k);
        let outside = 1.0 - ctl.space;
        let rumble = self.rumble.low(raw) * ctl.pressure * RUMBLE_LEVEL;
        let whoosh = self.whoosh.band(raw) * ctl.cloud * WHOOSH_LEVEL;
        engine + (rumble + whoosh) * outside
    }
}
