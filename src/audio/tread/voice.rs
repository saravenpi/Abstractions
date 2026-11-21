//! One footfall being heard: the heel strikes, the sole rolls, the toe lands and the foot lifts,
//! and the ground answers with its thud, its click, its grains, its ring and its wet sweep. A
//! landing strikes once with both feet, and a slide scrapes. What arrives is carried by the air
//! to the listener, and the listener's own step comes through the body as well.

use super::grain::Grains;
use super::material::{Material, of};
use super::sweep::Sweep;
use super::tune::{HEEL_TOE, SCRAPE, TAIL, TICK};
use crate::audio::cue::StepSurface;
use crate::audio::dsp::filter::{Lag, Svf};
use crate::audio::dsp::rand::Noise;
use crate::audio::ship::law::band_gain;
use crate::audio::ship::ring::Ringer;
use crate::audio::step::{Contact, Step};

/// What the pool works out of one step before a voice plays it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    /// Pitch factor of the ground, and of the thud alone, which the mass sets.
    pub pitch: (f32, f32),
    /// Level of the strike, the click, the ring and the wet sweep against a walk at home.
    pub push: f32,
    /// Grains per second and peak of a grain against a walk at home.
    pub grains: (f32, f32),
    /// Where the two resonances of the grains sit in the band of the ground, from 0 to 1.
    pub spread: f32,
    /// Samples until the sound arrives, until the toe lands and until the foot lifts.
    pub times: (u32, u32, u32),
    /// Gain of what the air carries, and of what the body carries.
    pub ears: (f32, f32),
    /// Left and right gains.
    pub sides: [f32; 2],
    /// Coefficient of the low-pass of the air on the way.
    pub dull: f32,
    /// Level of the tick of the armour, 0 for none.
    pub tick: f32,
}

/// One footfall voice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tread {
    /// What is being played, or nothing when the voice is free.
    step: Option<Step>,
    /// The ground under the foot.
    stuff: Material,
    /// What the pool worked out of the step.
    shape: Shape,
    /// Samples left to render, 0 when free.
    left: u32,
    /// The thud, the ring of the floor and the tick of the armour.
    solid: [Ringer; 3],
    /// Colour of the click.
    band: Svf,
    /// Level of the click now, the factor it dies by each sample, and its level at a strike.
    snap: (f32, f32, f32),
    /// The grains.
    grains: Grains,
    /// The wet sweep.
    wet: Sweep,
    /// The low-pass of the air.
    dull: Lag,
    /// Noise source.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Tread {
    /// Builds a free voice with its own noise `seed`.
    pub fn new(seed: u32, rate: f32) -> Self {
        let stuff = of(StepSurface::Grass);
        Self {
            step: None,
            stuff,
            shape: Shape {
                pitch: (1.0, 1.0),
                push: 0.0,
                grains: (0.0, 0.0),
                spread: 0.5,
                times: (0, 0, 0),
                ears: (0.0, 0.0),
                sides: [0.0; 2],
                dull: 1.0,
                tick: 0.0,
            },
            left: 0,
            solid: [Ringer::new(stuff.thud.0, stuff.thud.1, rate); 3],
            band: Svf::default(),
            snap: (0.0, 0.0, 0.0),
            grains: Grains::default(),
            wet: Sweep::default(),
            dull: Lag::default(),
            noise: Noise::new(seed),
            rate,
        }
    }

    /// Returns how many samples the voice still has to render: 0 when it is free.
    pub fn left(&self) -> u32 {
        self.left
    }

    /// Starts `step` with what the pool worked out of it.
    pub fn start(&mut self, step: &Step, shape: Shape) {
        let (stuff, rate) = (of(step.surface), self.rate);
        let ((pitch, thud), click) = (shape.pitch, stuff.click);
        self.solid[0].tune(stuff.thud.0 * thud, stuff.thud.1, rate);
        self.solid[1].tune(stuff.ring.0 * pitch, stuff.ring.1, rate);
        self.solid[2].tune(TICK.1 * pitch, TICK.2, rate);
        self.band = Svf::default();
        self.band.tune(click.0 * pitch, click.1, rate);
        let scrape = if step.contact == Contact::Skid {
            SCRAPE.0
        } else {
            click.2
        };
        let loud = band_gain(click.0 * pitch, click.1, rate) * click.3 * shape.push;
        self.snap = (0.0, (-1.0 / (scrape * rate)).exp(), loud);
        self.grains
            .tune(&stuff, (shape.spread, pitch), shape.grains, rate);
        self.wet = Sweep::default();
        self.dull = Lag::default();
        let tail = (stuff.grain.4 * 6.0).max(stuff.wet.2).max(TAIL);
        self.left = shape.times.0 + shape.times.1.max(shape.times.2) + (tail * rate) as u32;
        (self.step, self.stuff, self.shape) = (Some(*step), stuff, shape);
    }

    /// Lands a `share` of the step on the ground: the thud, the ring, the click and the grains.
    fn strike(&mut self, share: f32) {
        let (stuff, push) = (self.stuff, self.shape.push * share);
        self.solid[0].strike(stuff.thud.2 * push);
        self.solid[1].strike(stuff.ring.2 * push);
        self.snap.0 += self.snap.2 * share;
        self.grains.kick(share);
    }

    /// Starts the contact of the step: a strike of the heel, of both feet, or a scrape, and in
    /// mud or water the wet sweep of the foot going in.
    fn land(&mut self, contact: Contact) {
        let (stuff, pitch) = (self.stuff, self.shape.pitch.0);
        if stuff.wet.3 > 0.0 {
            let ends = (stuff.wet.0 * pitch, stuff.wet.1 * pitch);
            let level = stuff.wet.3 * self.shape.push;
            self.wet.start(ends, stuff.wet.2, level, self.rate);
        }
        match contact {
            Contact::Stride => self.strike(HEEL_TOE.0),
            Contact::Landing => {
                self.strike(1.0);
                self.solid[2].strike(self.shape.tick);
            }
            Contact::Skid => {
                self.snap.0 = self.snap.2;
                self.grains.hold(SCRAPE.0, self.rate);
                self.grains.kick(1.0);
            }
        }
    }

    /// Lifts the foot out of the ground: in mud it sucks, in water it drips.
    fn lift(&mut self) {
        let (stuff, pitch) = (self.stuff, self.shape.pitch.0);
        if stuff.wet.4 > 0.0 {
            let ends = (stuff.wet.1 * pitch, stuff.wet.0 * pitch);
            self.wet
                .start(ends, stuff.wet.2, stuff.wet.4 * self.shape.push, self.rate);
        }
    }

    /// Moves the step on by one sample: it arrives, the toe lands, the foot lifts.
    fn advance(&mut self) {
        let (arrive, toe, lift) = &mut self.shape.times;
        let due = [
            *arrive == 1,
            *arrive == 0 && *toe == 1,
            *arrive == 0 && *lift == 1,
        ];
        if *arrive > 0 {
            *arrive -= 1;
        } else {
            (*toe, *lift) = (toe.saturating_sub(1), lift.saturating_sub(1));
        }
        let contact = self.step.map_or(Contact::Stride, |step| step.contact);
        if due[0] {
            self.land(contact);
        }
        if due[1] && contact == Contact::Stride {
            self.strike(HEEL_TOE.1);
        }
        if due[2] {
            self.lift();
        }
    }

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        if self.left == 0 {
            return [0.0; 2];
        }
        self.left -= 1;
        self.advance();
        self.snap.0 *= self.snap.1;
        let (loose, grain) = self.grains.frame(&mut self.noise);
        self.solid[1].strike(grain * self.stuff.ring.3);
        let hard = self.band.band(self.noise.tick()) * self.snap.0;
        let wet = self.wet.frame(&mut self.noise, self.rate);
        let [thud, ring, tick] = &mut self.solid;
        let body = thud.frame() + ring.frame() + tick.frame();
        let (air, inner) = self.shape.ears;
        let heard = self
            .dull
            .to((body + hard + loose + wet) * air, self.shape.dull)
            + body * inner;
        let sides = self.shape.sides;
        [heard * sides[0], heard * sides[1]]
    }
}
