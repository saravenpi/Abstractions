//! The air around the listener: wind, sea wash and the muffled drone of being under water.

use super::dsp::filter::{Lag, Svf, pole};
use super::dsp::osc::{Phasor, sine};
use super::dsp::rand::Noise;
use super::shore::stereo;

/// Level of the wind at full strength.
pub const WIND_LEVEL: f32 = 0.4;

/// Level of the sea wash at the water's edge.
pub const SEA_LEVEL: f32 = 0.5;

/// Level of the drone under water.
pub const DEEP_LEVEL: f32 = 0.6;

/// Corner in hertz of the sea colour between two waves and what a breaking wave adds, for
/// water and for molten rock: lava is a rumble with nothing above the bass.
const WASH_HZ: [(f32, f32); 2] = [(450.0, 1500.0), (55.0, 85.0)];

/// How many times louder than waves the molten shore is driven: so narrow a band of noise
/// carries little, and the rumble would be lost under the wind.
const MOLTEN_GAIN: f32 = 3.0;

/// Seconds between two waves washing ashore.
const WAVE_SECONDS: f32 = 8.3;

/// Seconds for the wind tone to wander up and back.
const GUST_SECONDS: f32 = 11.0;

/// Corner in hertz of the outside sounds heard in the open.
const OPEN_HZ: f32 = 14_000.0;

/// Corner in hertz of the outside sounds heard from under water.
const MUFFLED_HZ: f32 = 350.0;

/// How much of the full muffling a closed cabin gives.
const CABIN_MUFFLE: f32 = 0.7;

/// What the air sounds follow, each from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AirCtl {
    /// Wind strength, already thinned by altitude.
    pub wind: f32,
    /// Shore closeness.
    pub shore: f32,
    /// Side the shore is on, from -1 (left) to 1 (right).
    pub shore_side: f32,
    /// Whether the shore is molten rock, which rumbles where water would wash.
    pub molten: bool,
    /// Depth under water.
    pub submerged: f32,
    /// 1 inside the ship cabin.
    pub cabin: f32,
    /// 1 in space, where nothing outside is heard.
    pub space: f32,
}

/// The ambience voices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Air {
    /// Wind colour, left and right.
    wind: [Svf; 2],
    /// Sea colour, left and right.
    sea: [Svf; 2],
    /// Under water drone colour.
    deep: Svf,
    /// Muffling of everything outside, left and right.
    muffle: [Lag; 2],
    /// Muffling coefficient.
    k: f32,
    /// Two independent noise sources.
    noise: [Noise; 2],
    /// Slow wander of the wind tone.
    gust: Phasor,
    /// The wave cycle.
    wave: Phasor,
    /// Sample rate in hertz.
    rate: f32,
}

/// Returns the level of a wave at `turn` of its cycle: a swell that never quite dies.
fn surge(turn: f32) -> f32 {
    let up = 0.5 + 0.5 * sine(turn.fract());
    0.12 + 0.88 * up * up
}

impl Air {
    /// Builds silent ambience for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            wind: [Svf::default(); 2],
            sea: [Svf::default(); 2],
            deep: Svf::default(),
            muffle: [Lag::default(); 2],
            k: 1.0,
            noise: [Noise::new(0x1F2E_3D4C), Noise::new(0x7A6B_5C4D)],
            gust: Phasor(0.0),
            wave: Phasor(0.0),
            rate,
        }
    }

    /// Retunes the filters. Call it every `dt` seconds, a few hundred times per second.
    pub fn tune(&mut self, ctl: &AirCtl, dt: f32) {
        let gust = sine(self.gust.tick(dt / GUST_SECONDS));
        let wave = self.wave.tick(dt / WAVE_SECONDS);
        for side in 0..2 {
            let lean = 1.0 + 0.2 * gust * (1.0 - 2.0 * side as f32);
            self.wind[side].tune((240.0 + 1000.0 * ctl.wind) * lean, 0.9, self.rate);
            let wash = surge(wave + 0.31 * side as f32);
            let (calm, breaking) = WASH_HZ[usize::from(ctl.molten)];
            self.sea[side].tune(calm + breaking * wash, 0.7, self.rate);
        }
        self.deep.tune(130.0, 0.8, self.rate);
        let closed = ctl.submerged.max(CABIN_MUFFLE * ctl.cabin);
        self.k = pole(OPEN_HZ * (MUFFLED_HZ / OPEN_HZ).powf(closed), self.rate);
    }

    /// Renders one stereo frame.
    pub fn frame(&mut self, ctl: &AirCtl) -> [f32; 2] {
        let raw = [self.noise[0].tick(), self.noise[1].tick()];
        let drone = self.deep.low(raw[0] + raw[1]) * ctl.submerged * DEEP_LEVEL;
        let blow = ctl.wind * ctl.wind.sqrt() * WIND_LEVEL;
        let outside = 1.0 - ctl.space;
        let drive = if ctl.molten { MOLTEN_GAIN } else { 1.0 };
        let sea = stereo(ctl.shore_side, ctl.shore * SEA_LEVEL * drive);
        let mut out = [0.0; 2];
        for side in 0..2 {
            let (low, band) = self.wind[side].run(raw[side]);
            let wash = surge(self.wave.0 + 0.31 * side as f32) * sea[side];
            let open = (low + 0.5 * band) * blow + self.sea[side].low(raw[1 - side]) * wash;
            out[side] = self.muffle[side].to(open * outside, self.k) + drone;
        }
        out
    }
}
