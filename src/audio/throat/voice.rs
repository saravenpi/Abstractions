//! One calling animal: its organ, its tract and the way to the ear, sample by sample.
//!
//! The source is brought to a steady root mean square before its envelope, so a voice whose
//! harmonics miss the formants is as loud as one whose harmonics hit them; the envelope, the trill
//! and the distance then set its level.

use super::folds::{Folds, Push};
use super::notes::Note;
use super::plan::Plan;
use super::pulses::Pulses;
use super::shape::Source;
use super::tract::Tract;
use super::tune::{BODY_RMS, FOLLOW, RETUNE};
use crate::audio::dsp::filter::glide;
use crate::audio::dsp::osc::{Phasor, sine};
use crate::audio::dsp::rand::Noise;
use crate::audio::spatial::{Ear, Spot};

/// Lowest root mean square the level follower divides by.
const FLOOR: f32 = 1e-3;

/// Level of the second harmonic of a syrinx against its note.
const SYRINX_SECOND: f32 = 0.12;

/// One voice of an animal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Throat {
    /// The call it sounds.
    pub plan: Plan,
    /// Whether it sounds.
    live: bool,
    /// Seconds into the call.
    pub at: f32,
    /// Sample rate in hertz.
    rate: f32,
    /// Per-sample coefficient of the level follower.
    follow: f32,
    /// The vocal folds.
    folds: Folds,
    /// The file.
    pulses: Pulses,
    /// Phases of the syrinx, the vibrato and the trill.
    osc: [Phasor; 3],
    /// The tract and the way to the ear.
    tract: Tract,
    /// Breath and scrape.
    noise: Noise,
    /// Mean square of the shaped source, followed.
    power: f32,
    /// The two ears: where the call comes from.
    pub ear: Ear,
    /// Samples since the start, to retune now and then.
    tick: u32,
}

impl Throat {
    /// Builds a silent voice with its own noise, for a sample `rate`.
    pub fn new(seed: u32, rate: f32, plan: Plan) -> Self {
        Self {
            plan,
            live: false,
            at: 0.0,
            rate,
            follow: glide(FOLLOW, rate),
            folds: Folds::new(0.5),
            pulses: Pulses::default(),
            osc: [Phasor(0.0); 3],
            tract: Tract::default(),
            noise: Noise::new(seed),
            power: 0.0,
            ear: Ear::new(1.0),
            tick: 0,
        }
    }

    /// Returns true when the voice is free.
    pub fn idle(&self) -> bool {
        !self.live
    }

    /// Starts `plan`, heard as `spot` says and as late, its chaos seeded by `seed`, 0 to 1.
    pub fn start(&mut self, plan: Plan, spot: &Spot, seed: f32) {
        let open = plan.source == Source::Syrinx;
        self.tract
            .tune((plan.formants, open), plan.muffled, self.rate);
        self.pulses.reset(plan.hz, plan.teeth.1, self.rate);
        self.folds = Folds::new(seed);
        self.ear.place(spot, self.rate);
        (self.plan, self.live) = (plan, true);
        (self.at, self.power, self.tick) = (-spot.delay, 0.0, 0);
    }

    /// Returns the pitch factor of the vibrato now.
    fn wobble(&mut self) -> f32 {
        let (depth, hz) = self.plan.vibrato;
        1.0 + depth * sine(self.osc[1].tick(hz / self.rate))
    }

    /// Returns the level of the trill now, 1 for a voice with none.
    fn trill(&mut self) -> f32 {
        match self.plan.trill > 0.0 {
            true => 0.5 + 0.5 * sine(self.osc[2].tick(self.plan.trill / self.rate)),
            false => 1.0,
        }
    }

    /// Returns one sample of the file at `hz`, its teeth and its ring following the pitch.
    fn scrape(&mut self, hz: f32) -> f32 {
        if self.tick.is_multiple_of(RETUNE) {
            self.pulses.tune(hz, self.plan.teeth.1, self.rate);
        }
        let teeth = self.plan.teeth.0 * hz / self.plan.hz.max(1.0);
        self.pulses.tick(teeth, self.rate, &mut self.noise)
    }

    /// Returns one sample of the source for `note`, `u` into it.
    fn excite(&mut self, note: &Note, u: f32) -> f32 {
        let hz = self.plan.hz * note.pitch(u) * self.wobble();
        match (self.plan.source, note.voiced) {
            (Source::File, _) => self.scrape(hz),
            (Source::Syrinx, true) => {
                let turn = self.osc[0].tick(hz / self.rate);
                sine(turn) + SYRINX_SECOND * sine((2.0 * turn).fract())
            }
            (Source::Larynx, true) => {
                let push = Push {
                    hz,
                    rough: note.rough(u),
                    breath: self.plan.breath,
                    second: self.plan.second,
                };
                self.folds.tick(&push, self.rate, &mut self.noise)
            }
            _ => self.noise.tick(),
        }
    }

    /// Renders one sample.
    pub fn frame(&mut self) -> f32 {
        if !self.live {
            return 0.0;
        }
        let t = self.at;
        self.at += 1.0 / self.rate;
        self.tick = self.tick.wrapping_add(1);
        if t >= self.plan.seconds() {
            self.live = false;
            return 0.0;
        }
        let (x, envelope) = match self.plan.at(t) {
            Some((note, u)) => (self.excite(&note, u), note.envelope(u)),
            None => (0.0, 0.0),
        };
        let shaped = self.tract.shape(x);
        self.power += self.follow * (shaped * shaped - self.power);
        let level = BODY_RMS / self.power.sqrt().max(FLOOR);
        let voice = shaped * level * envelope * self.trill() * self.plan.gain;
        self.tract.carry(voice)
    }
}
