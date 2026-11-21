//! The way from a ship to a listener outside it: the sound takes `distance / c` seconds to
//! arrive, spreads as `1 / r`, loses its high end to the air, meets its own reflection off the
//! ground, and comes from one side.
//!
//! The delay is a real delay line read at a moving point. A ship that closes in shortens it,
//! so its sound is played faster: the Doppler shift needs nothing more.

use super::feed::{Ch, HullSound};
use super::law::{absorb_hz, spread};
use super::tune::{BOUNCE, DELAY_GLIDE, MAX_PACE, NEAR, TRAVEL};
use crate::audio::dsp::filter::{Lag, glide, pole};
use crate::audio::shore::stereo;

/// What the path is on its way to.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Goal {
    /// Delay of the direct sound in samples.
    delay: f32,
    /// Samples the reflection off the ground arrives after the direct sound.
    gap: f32,
    /// Coefficient of the low-pass of the air.
    dull: f32,
    /// Left and right gains.
    sides: [f32; 2],
}

/// The path of the sound of one ship through the air.
#[derive(Debug, Clone)]
pub struct Path {
    /// The sound on its way, allocated once.
    line: Vec<f32>,
    /// Where the next sample is written.
    at: usize,
    /// The delay, through two stages of smoothing.
    delay: [Lag; 2],
    /// The lateness of the reflection.
    gap: Lag,
    /// The low-pass of the air.
    dull: Lag,
    /// Left and right gains.
    sides: [Lag; 2],
    /// What all of these move toward.
    goal: Goal,
    /// Coefficient of the delay smoothing.
    k: f32,
    /// Sample rate in hertz.
    rate: f32,
}

impl Path {
    /// Builds an empty path.
    pub fn new(rate: f32) -> Self {
        Self {
            line: vec![0.0; (TRAVEL * rate) as usize + 4],
            at: 0,
            delay: [Lag(1.0); 2],
            gap: Lag::default(),
            dull: Lag::default(),
            sides: [Lag::default(); 2],
            goal: Goal::default(),
            k: glide(DELAY_GLIDE, rate),
            rate,
        }
    }

    /// Returns the longest delay the line holds, in samples.
    fn most(&self) -> f32 {
        (self.line.len() - 3) as f32
    }

    /// Aims the path at where `hull` is for the listener. Call it a few hundred times per second.
    pub fn tune(&mut self, hull: &HullSound) {
        let (far, sound) = (hull.at(Ch::Distance), hull.at(Ch::Sound));
        let high = hull.at(Ch::Height) + BOUNCE.1;
        let late = 2.0 * BOUNCE.0 * high / (far.max(NEAR.0) * sound);
        self.goal = Goal {
            delay: (far / sound * self.rate).clamp(1.0, self.most()),
            gap: (late * self.rate).min(0.05 * self.rate),
            dull: pole(absorb_hz(far, hull.at(Ch::Air)), self.rate),
            sides: stereo(hull.at(Ch::Side), spread(far)),
        };
    }

    /// Empties the line and jumps to where `hull` is: for a ship that has just appeared.
    pub fn clear(&mut self, hull: &HullSound) {
        self.line.fill(0.0);
        self.tune(hull);
        self.delay = [Lag(self.goal.delay); 2];
        self.gap = Lag(self.goal.gap);
        self.dull = Lag::default();
        self.sides = self.goal.sides.map(Lag);
    }

    /// Sends one sample of the ship on its way.
    pub fn write(&mut self, sample: f32) {
        self.line[self.at] = sample;
        self.at = (self.at + 1) % self.line.len();
    }

    /// Returns the sound that left the ship `delay` samples ago.
    fn tap(&self, delay: f32) -> f32 {
        let len = self.line.len();
        let whole = delay as usize;
        let part = delay - whole as f32;
        let near = (self.at + len - 1 - whole) % len;
        let far = (near + len - 1) % len;
        self.line[near] * (1.0 - part) + self.line[far] * part
    }

    /// Renders one stereo frame of what arrives at the listener now.
    pub fn read(&mut self) -> [f32; 2] {
        let first = self.delay[0].to(self.goal.delay, self.k);
        let before = self.delay[1].0;
        let wanted = self.delay[1].to(first, self.k);
        let delay = wanted.clamp(before - MAX_PACE, before + MAX_PACE).max(1.0);
        self.delay[1].0 = delay;
        let gap = self.gap.to(self.goal.gap, self.k);
        let late = (delay + gap).min(self.most());
        let heard = self.tap(delay) + BOUNCE.2 * self.tap(late);
        let soft = self.dull.to(heard, self.goal.dull);
        let goal = self.goal.sides;
        let k = self.k;
        [0, 1].map(|side| soft * self.sides[side].to(goal[side], k))
    }
}
