//! The flight gear: the rush of air past the body, the rumble of its buffeting, the hum of the
//! boot boosters and the flutter of the wingsuit cloth.

use super::dsp::filter::Svf;
use super::dsp::osc::{Phasor, sine};
use super::dsp::rand::Noise;

/// Level of the rush of air at full speed.
pub const RUSH_LEVEL: f32 = 0.5;

/// Level of the rumble at full buffeting.
pub const BUFFET_LEVEL: f32 = 0.9;

/// Level of the boosters at full push.
pub const HUM_LEVEL: f32 = 0.07;

/// Level of the cloth at full flutter.
pub const FLUTTER_LEVEL: f32 = 0.25;

/// Booster pitch in hertz at the lightest push, and what full push adds.
const HUM_HZ: (f32, f32) = (62.0, 46.0);

/// Flaps per second of the cloth at the lightest flutter, and what full flutter adds.
const FLAP_HZ: (f32, f32) = (11.0, 17.0);

/// What the gear sounds follow, each from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuitCtl {
    /// How loud the rush of air is: the speed of the air past the body, thinned by the air.
    pub rush: f32,
    /// Speed of the air past the body, whatever the air: the pitch of the rush follows it.
    pub speed: f32,
    /// Buffeting of the air on the body, from its dynamic pressure.
    pub buffet: f32,
    /// Push of the boot boosters.
    pub boost: f32,
    /// Flutter of the wingsuit cloth.
    pub flutter: f32,
}

/// The gear voices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Suit {
    /// Colour of the rush: it brightens with speed.
    rush: Svf,
    /// Colour of the rumble of the buffeting: low, and a little higher as it grows.
    rumble: Svf,
    /// Colour of the booster breath: a low, wide band, so the boots rumble and never whistle.
    hiss: Svf,
    /// Colour of the cloth.
    cloth: Svf,
    /// Booster tone: fundamental and a beating fifth.
    hum: [Phasor; 2],
    /// The flaps of the cloth.
    flap: Phasor,
    /// Noise source.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Suit {
    /// Builds silent gear for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            rush: Svf::default(),
            rumble: Svf::default(),
            hiss: Svf::default(),
            cloth: Svf::default(),
            hum: [Phasor(0.0), Phasor(0.37)],
            flap: Phasor(0.0),
            noise: Noise::new(0x6C0F_F1E5),
            rate,
        }
    }

    /// Retunes the filters. Call it a few hundred times per second.
    pub fn tune(&mut self, ctl: &SuitCtl) {
        self.rush.tune(280.0 + 2400.0 * ctl.speed, 0.8, self.rate);
        self.rumble.tune(48.0 + 60.0 * ctl.buffet, 1.0, self.rate);
        self.hiss.tune(260.0 + 240.0 * ctl.boost, 0.6, self.rate);
        self.cloth.tune(700.0 + 500.0 * ctl.flutter, 1.4, self.rate);
    }

    /// Renders one sample.
    pub fn frame(&mut self, ctl: &SuitCtl) -> f32 {
        let raw = self.noise.tick();
        let (low, band) = self.rush.run(raw);
        let rush = (low + 0.6 * band) * ctl.rush * ctl.rush.sqrt() * RUSH_LEVEL;
        let step = (HUM_HZ.0 + HUM_HZ.1 * ctl.boost) / self.rate;
        let [root, fifth] = &mut self.hum;
        let tone = sine(root.tick(step)) + 0.5 * sine(fifth.tick(step * 1.498));
        let hum = (tone + 0.9 * self.hiss.band(raw)) * ctl.boost * HUM_LEVEL;
        let flap = sine(
            self.flap
                .tick((FLAP_HZ.0 + FLAP_HZ.1 * ctl.flutter) / self.rate),
        );
        let beat = (0.5 + 0.5 * flap) * (0.5 + 0.5 * flap);
        let cloth = self.cloth.band(raw) * beat * ctl.flutter * FLUTTER_LEVEL;
        let rumble = self.rumble.low(raw) * ctl.buffet * BUFFET_LEVEL;
        rush + rumble + hum + cloth
    }
}
