//! The master bus: every voice summed, gains smoothed, output guarded.

use std::sync::Arc;

use super::ambience::{Air, AirCtl};
use super::cues::Pool;
use super::dsp::filter::{Lag, glide, pole};
use super::dsp::limit::{DcBlock, limit};
use super::music::{Music, MusicCtl};
use super::shared::{PARAMS, Param, Shared};
use super::ship::{Ship, ShipCtl};
use super::suit::{Suit, SuitCtl};

/// Frames between two readings of the shared values.
pub const BLOCK: u32 = 32;

/// Seconds for a gain or a scene value to cover 63 percent of a change. This removes zipper noise.
pub const SMOOTH_SECONDS: f32 = 0.04;

/// Level of the music bus at full music volume.
pub const MUSIC_GAIN: f32 = 1.0;

/// Level of the ambience, ship and cue bus at full effects volume.
pub const EFFECTS_GAIN: f32 = 1.0;

/// Corner in hertz of the filter that removes any constant offset.
const DC_HZ: f32 = 12.0;

/// The whole synthesiser. Rendering a frame never allocates and never locks.
#[derive(Debug)]
pub struct Mixer {
    /// Values written by the game.
    shared: Arc<Shared>,
    /// Sample rate in hertz.
    rate: f32,
    /// Last values read, one per [`Param`].
    targets: [f32; PARAMS],
    /// Smoothed values. The master volume starts at zero so that the output fades in.
    now: [Lag; PARAMS],
    /// Music engine.
    music: Music,
    /// Ambience voices.
    air: Air,
    /// Ship voices.
    ship: Ship,
    /// Flight gear voices.
    suit: Suit,
    /// One-shot voices.
    cues: Pool,
    /// Offset removal, left and right.
    dc: [DcBlock; 2],
    /// Frames rendered in the current block.
    count: u32,
    /// Per-sample coefficient of the value smoothing.
    k: f32,
    /// Pole of the offset removal.
    r: f32,
}

impl Mixer {
    /// Builds the synthesiser for an output sample rate, silent.
    pub fn new(rate: f32, shared: Arc<Shared>) -> Self {
        let music = Music::new(rate, shared.seed(), shared.get(Param::Home) > 0.5);
        let mut now = Param::ALL.map(|param| Lag(shared.get(param)));
        now[Param::Master as usize] = Lag(0.0);
        Self {
            shared,
            rate,
            targets: [0.0; PARAMS],
            now,
            music,
            air: Air::new(rate),
            ship: Ship::new(rate),
            suit: Suit::new(rate),
            cues: Pool::new(rate),
            dc: [DcBlock::default(); 2],
            count: 0,
            k: glide(SMOOTH_SECONDS, rate),
            r: 1.0 - pole(DC_HZ, rate),
        }
    }

    /// Returns the smoothed value of a parameter.
    fn at(&self, param: Param) -> f32 {
        self.now[param as usize].0
    }

    /// Returns what the ambience follows.
    fn air_ctl(&self) -> AirCtl {
        AirCtl {
            wind: self.at(Param::Wind),
            shore: self.at(Param::Shore),
            shore_side: self.at(Param::ShoreSide) * 2.0 - 1.0,
            submerged: self.at(Param::Submerged),
            cabin: self.at(Param::Cabin),
            space: self.at(Param::Space),
        }
    }

    /// Returns what the ship follows.
    fn ship_ctl(&self) -> ShipCtl {
        ShipCtl {
            throttle: self.at(Param::Throttle),
            pressure: self.at(Param::Pressure),
            cloud: self.at(Param::Cloud),
            cabin: self.at(Param::Cabin),
            space: self.at(Param::Space),
        }
    }

    /// Returns what the flight gear follows.
    fn suit_ctl(&self) -> SuitCtl {
        SuitCtl {
            rush: self.at(Param::Rush),
            boost: self.at(Param::Boost),
            flutter: self.at(Param::Flutter),
        }
    }

    /// Reads the shared values, starts waiting cues and retunes every filter.
    fn control(&mut self) {
        for param in Param::ALL {
            self.targets[param as usize] = self.shared.get(param);
        }
        let buddy = self.targets[Param::BuddySide as usize] * 2.0 - 1.0;
        self.cues
            .locate(buddy, self.targets[Param::BuddyNear as usize]);
        while let Some(cue) = self.shared.pop() {
            self.cues.play(cue);
        }
        let dt = BLOCK as f32 / self.rate;
        let orders = MusicCtl {
            seed: self.shared.seed(),
            home: self.targets[Param::Home as usize] > 0.5,
            level: self.targets[Param::MusicLevel as usize],
            day: self.targets[Param::Day as usize],
            submerged: self.targets[Param::Submerged as usize],
        };
        self.music.steer(&orders, dt);
        self.air.tune(&self.air_ctl(), dt);
        self.ship.tune(&self.ship_ctl());
        self.suit.tune(&self.suit_ctl());
    }

    /// Renders one stereo frame, always within full scale.
    pub fn frame(&mut self) -> [f32; 2] {
        if self.count == 0 {
            self.control();
        }
        self.count = (self.count + 1) % BLOCK;
        for (now, target) in self.now.iter_mut().zip(self.targets) {
            now.settle(target, self.k);
        }
        let music = self.music.frame();
        let air = self.air.frame(&self.air_ctl());
        let gear = self.suit.frame(&self.suit_ctl());
        let mono = self.ship.frame(&self.ship_ctl()) + gear;
        let cues = self.cues.frame();
        let (wide, fx) = (
            self.at(Param::Music) * MUSIC_GAIN,
            self.at(Param::Effects) * EFFECTS_GAIN,
        );
        let (master, r) = (self.at(Param::Master), self.r);
        [0, 1].map(|i| {
            let sum = music[i] * wide + (air[i] + mono + cues[i]) * fx;
            limit(self.dc[i].run(sum * master, r))
        })
    }
}
