//! The master bus: every voice summed, gains smoothed, output guarded.

use std::sync::Arc;

use super::ambience::{Air, AirCtl};
use super::cues::Pool;
use super::dsp::filter::{Lag, glide};
use super::master::{Duck, Master, Output};
use super::music::{Music, MusicCtl};
use super::planet::class;
use super::scene::TITLE_SEED;
use super::shared::{PARAMS, Param, Shared};
use super::ship::Fleet;
use super::spatial::{Absorb, Seat};
use super::suit::{Suit, SuitCtl};
use super::tread::Treads;
use super::tune::{BED_MUSIC, CUES_GAIN, PAUSE_FADE, VIEW_FADE};
use crate::universe::PlanetClass;

/// Frames between two readings of the shared values.
pub const BLOCK: u32 = 32;

/// Seconds for a gain or a scene value to cover 63 percent of a change. This removes zipper noise.
pub const SMOOTH_SECONDS: f32 = 0.04;

/// Level of the music bus at full music volume.
pub const MUSIC_GAIN: f32 = 1.0;

/// Level of the ambience, ship and cue bus at full effects volume.
pub const EFFECTS_GAIN: f32 = 1.0;

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
    fleet: Fleet,
    /// The seat and the cabin around it.
    seat: Seat,
    /// Flight gear voices.
    suit: Suit,
    /// One-shot voices.
    cues: Pool,
    /// Footfall voices.
    treads: Treads,
    /// How the music gives way to the effects.
    duck: Duck,
    /// The master bus.
    master: Master,
    /// Frames rendered in the current block.
    count: u32,
    /// Per-sample coefficient of the smoothing of each value: the move of the ears into the cabin
    /// and out is slower than the rest.
    k: [f32; PARAMS],
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
            fleet: Fleet::new(rate, BLOCK as f32 / rate),
            seat: Seat::new(rate),
            suit: Suit::new(rate),
            cues: Pool::new(rate),
            treads: Treads::new(rate),
            duck: Duck::new(rate),
            master: Master::new(rate),
            count: 0,
            k: Param::ALL.map(|param| match param {
                Param::Cabin => glide(VIEW_FADE, rate),
                Param::World => glide(PAUSE_FADE, rate),
                _ => glide(SMOOTH_SECONDS, rate),
            }),
        }
    }

    /// Returns the smoothed value of a parameter.
    fn at(&self, param: Param) -> f32 {
        self.now[param as usize].0
    }

    /// Returns what the ambience follows.
    fn air_ctl(&self) -> AirCtl {
        let world = class(self.targets[Param::Class as usize]);
        AirCtl {
            wind: self.at(Param::Wind),
            shore: self.at(Param::Shore),
            shore_side: self.at(Param::ShoreSide) * 2.0 - 1.0,
            molten: world == PlanetClass::Lava,
            deep: world == PlanetClass::GasGiant,
            submerged: self.at(Param::Submerged),
            cabin: self.at(Param::Cabin),
            space: self.at(Param::Space),
            rain: self.at(Param::Rain),
            dust: self.at(Param::Dust),
        }
    }

    /// Returns what the flight gear follows.
    fn suit_ctl(&self) -> SuitCtl {
        SuitCtl {
            rush: self.at(Param::Rush),
            boost: self.at(Param::Boost),
            flutter: self.at(Param::Flutter),
            speed: self.at(Param::Speed),
            buffet: self.at(Param::Buffet),
        }
    }

    /// Returns how the air around the listener takes the high end with distance.
    fn absorb(&self) -> Absorb {
        Absorb {
            far: self.targets[Param::AirFar as usize],
            slope: self.targets[Param::AirSlope as usize],
        }
    }

    /// Renders one stereo frame of every effect at the effects volume: the ambience, the flight
    /// gear, the ships from the seat or outside, the one-shot sounds and the steps, all of them at
    /// the level of the world, which a pause takes to silence, but for the interface.
    fn effects(&mut self) -> [f32; 2] {
        let air = self.air.frame(&self.air_ctl());
        let gear = self.suit.frame(&self.suit_ctl());
        let (blips, steps) = (self.cues.frame(), self.treads.frame());
        let ((sent, ui), cabin) = (self.cues.sent(), self.at(Param::Cabin));
        let ships = self.seat.frame(self.fleet.frame(), sent, cabin);
        let (fx, world) = (
            self.at(Param::Effects) * EFFECTS_GAIN,
            self.at(Param::World),
        );
        [0, 1].map(|i| {
            let sounds = air[i] + gear + ships[i] + (blips[i] - ui[i] + steps[i]) * CUES_GAIN;
            (sounds * world + ui[i] * CUES_GAIN) * fx
        })
    }

    /// Reads the shared values, starts waiting cues and retunes every filter.
    fn control(&mut self) {
        for param in Param::ALL {
            self.targets[param as usize] = self.shared.get(param);
        }
        let still = self.targets[Param::World as usize] < 0.5;
        while let Some((cue, spot)) = self.shared.pop() {
            if still && !cue.is_interface() {
                continue;
            }
            self.fleet.hear(cue);
            self.treads.hear(cue, &spot);
            self.cues.play(cue, &spot);
        }
        let dt = BLOCK as f32 / self.rate;
        let seed = self.shared.seed();
        let bed = if seed == TITLE_SEED { 1.0 } else { BED_MUSIC };
        let orders = MusicCtl {
            seed,
            home: self.targets[Param::Home as usize] > 0.5,
            level: self.targets[Param::MusicLevel as usize] * bed,
            day: self.targets[Param::Day as usize],
            submerged: self.targets[Param::Submerged as usize],
        };
        self.music.dress(class(self.targets[Param::Class as usize]));
        self.music.steer(&orders, dt);
        self.air.tune(&self.air_ctl(), dt);
        let cloud = self.at(Param::Cloud) * (1.0 - self.at(Param::Space));
        self.fleet
            .tune(&self.shared.hulls, (cloud, self.absorb()), dt);
        self.suit.tune(&self.suit_ctl());
        self.master.tune();
    }

    /// Renders one stereo frame, always within full scale.
    pub fn frame(&mut self) -> [f32; 2] {
        if self.count == 0 {
            self.control();
        }
        self.count = (self.count + 1) % BLOCK;
        for ((now, target), k) in self.now.iter_mut().zip(self.targets).zip(self.k) {
            now.settle(target, k);
        }
        let music = self.music.frame();
        let effects = self.effects();
        let wide = self.at(Param::Music) * MUSIC_GAIN;
        let under = self.duck.gain(effects) * wide;
        let output = Output {
            volume: self.at(Param::Master),
            reduced: self.at(Param::Reduced),
            mono: self.at(Param::Mono),
        };
        self.master
            .run([0, 1].map(|i| music[i] * under + effects[i]), &output)
    }
}
