//! One footfall being heard: the heel strikes, the sole rolls, the toe lands, and the ground
//! answers each of the two with its thud, its click, its grains and its ring.
//!
//! The grains follow a physically informed stochastic model: after an impact the ground holds
//! an energy that dies away, and each grain is a chance event whose odds follow that energy. No
//! two steps draw the same grains.

use super::material::{Material, of};
use crate::audio::dsp::filter::Svf;
use crate::audio::dsp::rand::Noise;
use crate::audio::ship::law::band_gain;
use crate::audio::ship::ring::Ringer;
use crate::audio::step::Step;

/// Share of a step the heel takes, and the share the toe takes after it.
const HEEL_TOE: (f32, f32) = (0.6, 0.4);

/// Seconds from heel to toe at the lightest and at the hardest step: a slow walk rolls, a run slaps.
const ROLL: (f32, f32) = (0.13, 0.03);

/// Resonance of the thud.
const THUD_Q: f32 = 2.5;

/// Share of the click and the grains, and of the thud, heard through the body alone.
const THROUGH_BODY: (f32, f32) = (0.08, 0.6);

/// How a step differs from the plain one of its ground: its pitch and its level as factors, and
/// its left and right gains.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turn {
    /// Pitch factor.
    pub pitch: f32,
    /// Level factor.
    pub level: f32,
    /// Left and right gains.
    pub sides: [f32; 2],
}

/// One footfall voice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tread {
    /// Samples left to render, 0 when free.
    left: u32,
    /// Samples until the toe lands.
    toe: u32,
    /// The ground under the foot.
    stuff: Material,
    /// How hard the step is, with its own level.
    force: f32,
    /// The thud and the ring.
    solid: [Ringer; 2],
    /// Colours of the click and of the grains.
    bands: [Svf; 2],
    /// Level of the click now, the factor it dies by each sample, and its level at an impact.
    snap: (f32, f32, f32),
    /// Energy of the grains now and the factor it dies by each sample.
    crumble: (f32, f32),
    /// Odds of a grain per sample at full energy, and the level of the grains.
    grains: (f32, f32),
    /// Noise source.
    noise: Noise,
    /// Left and right gains.
    sides: [f32; 2],
}

impl Tread {
    /// Builds a free voice with its own noise `seed`.
    pub fn new(seed: u32, rate: f32) -> Self {
        Self {
            left: 0,
            toe: 0,
            stuff: of(Step::on(crate::audio::cue::StepSurface::Sand).surface),
            force: 0.0,
            solid: [Ringer::new(80.0, THUD_Q, rate); 2],
            bands: [Svf::default(); 2],
            snap: (0.0, 0.0, 0.0),
            crumble: (0.0, 0.0),
            grains: (0.0, 0.0),
            noise: Noise::new(seed),
            sides: [1.0; 2],
        }
    }

    /// Returns how many samples the voice still has to render: 0 when it is free.
    pub fn left(&self) -> u32 {
        self.left
    }

    /// Starts `step` with its `turn`, at a sample rate `rate`.
    pub fn start(&mut self, step: &Step, turn: &Turn, rate: f32) {
        let stuff = of(step.surface);
        let (air, body) = if step.body { THROUGH_BODY } else { (1.0, 1.0) };
        let (click, grain) = (stuff.click, stuff.grain);
        let odds = (grain.2 / rate).clamp(1.0e-6, 0.5);
        self.stuff = stuff;
        self.force = step.force * turn.level * body;
        self.solid[0].tune(stuff.thud.0 * turn.pitch, THUD_Q, rate);
        self.solid[1].tune(stuff.ring.0 * turn.pitch, stuff.ring.1, rate);
        self.bands[0].tune(click.0 * turn.pitch, click.1, rate);
        self.bands[1].tune(grain.0 * turn.pitch, grain.1, rate);
        let loud = band_gain(click.0, click.1, rate) * click.3 * air / body;
        self.snap = (0.0, (-1.0 / (click.2 * rate)).exp(), loud);
        self.crumble = (0.0, (-1.0 / (grain.3 * rate)).exp());
        let level = band_gain(grain.0, grain.1, rate) / (odds / 3.0).sqrt() * grain.4;
        self.grains = (odds, level * self.force * air / body);
        let roll = ROLL.0 + (ROLL.1 - ROLL.0) * step.force.clamp(0.0, 1.0);
        self.toe = if step.landing { 0 } else { (roll * rate) as u32 };
        self.left = self.toe + (4.0 * grain.3.max(0.1) * rate) as u32;
        self.sides = turn.sides;
        self.strike(if step.landing { 1.0 } else { HEEL_TOE.0 });
    }

    /// Lands a `share` of the step on the ground: the thud, the ring, the click and the grains.
    fn strike(&mut self, share: f32) {
        let push = self.force * share;
        self.solid[0].strike(self.stuff.thud.1 * push);
        self.solid[1].strike(self.stuff.ring.2 * push);
        self.snap.0 += self.snap.2 * push;
        self.crumble.0 = (self.crumble.0 + share).min(1.0);
    }

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        if self.left == 0 {
            return [0.0; 2];
        }
        if self.toe == 1 {
            self.strike(HEEL_TOE.1);
        }
        self.toe = self.toe.saturating_sub(1);
        self.left -= 1;
        self.snap.0 *= self.snap.1;
        self.crumble.0 *= self.crumble.1;
        let odds = 2.0 * self.grains.0 * self.crumble.0;
        let hit = (self.noise.tick() - (1.0 - odds)).max(0.0) / odds.max(1.0e-9);
        let [click, grain] = &mut self.bands;
        let loose = grain.band(hit) * self.grains.1 * self.crumble.0.sqrt();
        let hard = click.band(self.noise.tick()) * self.snap.0;
        let [thud, ring] = &mut self.solid;
        let sum = thud.frame() + ring.frame() + hard + loose;
        [sum * self.sides[0], sum * self.sides[1]]
    }
}
