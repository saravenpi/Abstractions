//! The way from a ship to a listener outside it: the sound takes `distance / c` seconds to
//! arrive, spreads as `1 / r`, loses its high end to the air of the planet, meets its own
//! reflection off the ground, and reaches the two ears through the shared [`Ear`].
//!
//! The delay is a real delay line read at a moving point. A ship that closes in shortens it,
//! so its sound is played faster: that is the Doppler shift. What is heard now left the ship
//! where it was then, not where it is now, so the delay is the distance over `c - v`, with `v`
//! the speed the ship closes in at: a tone is raised by `c / (c - v)`.

use super::feed::{Ch, HullSound};
use super::law::spread;
use super::tune::{BOUNCE, DELAY_GLIDE, MAX_PACE, NEAR, RIDE_FOLLOW, TRAVEL};
use crate::audio::dsp::filter::{Lag, glide, pole};
use crate::audio::mix::BLOCK;
use crate::audio::spatial::{Absorb, Ear, Spot, around};

/// Seconds the closing speed of a ship is averaged over. Long, because the delay of a far ship
/// is large and any ripple of its speed would shake it.
const SPEED_GLIDE: f32 = 0.5;

/// What the path is on its way to.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Goal {
    /// Delay of the direct sound in samples.
    delay: f32,
    /// Samples the reflection off the ground arrives after the direct sound.
    gap: f32,
    /// Coefficient of the low-pass of the air.
    dull: f32,
    /// How the sound reaches the ears, but for the delay and the air the path makes itself.
    spot: Spot,
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
    /// How fast the delay of the distance changes, in samples per sample: minus the closing
    /// speed over the speed of sound, and its coefficient.
    slope: (Lag, f32),
    /// The lateness of the reflection.
    gap: Lag,
    /// The low-pass of the air.
    dull: Lag,
    /// The two ears.
    ear: Ear,
    /// The distance to a ship the listener rides along with, followed slowly, and the coefficient
    /// of one tuning.
    follow: (Lag, f32),
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
            slope: (Lag::default(), glide(SPEED_GLIDE, rate)),
            gap: Lag::default(),
            dull: Lag::default(),
            ear: Ear::new(glide(DELAY_GLIDE, rate)),
            follow: (Lag::default(), glide(RIDE_FOLLOW, rate / BLOCK as f32)),
            goal: Goal {
                delay: 1.0,
                gap: 0.0,
                dull: 0.0,
                spot: Spot::FLAT,
            },
            k: glide(DELAY_GLIDE, rate),
            rate,
        }
    }

    /// Returns the longest delay the line holds, in samples.
    fn most(&self) -> f32 {
        (self.line.len() - 3) as f32
    }

    /// Aims the path at where `hull` is for the listener, in air that takes the high end as
    /// `absorb` says. Call it once a block. The distance to a ship the listener rides along with
    /// follows slowly, so the sway of the chase camera is heard as neither a pulse nor a Doppler.
    pub fn tune(&mut self, hull: &HullSound, absorb: &Absorb) {
        let ride = if hull.at(Ch::Riding) > 0.5 {
            self.follow.1
        } else {
            1.0
        };
        let far = self.follow.0.to(hull.at(Ch::Distance), ride);
        let sound = hull.at(Ch::Sound);
        let high = hull.at(Ch::Height) + BOUNCE.1;
        let late = 2.0 * BOUNCE.0 * high / (far.max(NEAR.0) * sound);
        let side = hull.at(Ch::Side);
        self.goal = Goal {
            delay: (far / sound * self.rate).clamp(1.0, self.most()),
            gap: (late * self.rate).min(0.05 * self.rate),
            dull: pole(absorb.corner(far), self.rate),
            spot: Spot {
                air: spread(far),
                side,
                front: hull.at(Ch::Front),
                gap: around(side, sound),
                ..Spot::FLAT
            },
        };
        self.ear.aim(&self.goal.spot, self.rate);
    }

    /// Jumps to where `hull` is and keeps the sound already on its way: for a listener who has
    /// just got out, while the outside is still silent. Gliding there instead would sweep the
    /// delay through the seat and play the sound on its way at twice its pitch.
    pub fn jump(&mut self, hull: &HullSound, absorb: &Absorb) {
        self.follow.0 = Lag(hull.at(Ch::Distance));
        self.tune(hull, absorb);
        self.delay = [Lag(self.goal.delay); 2];
        self.slope.0 = Lag::default();
        self.gap = Lag(self.goal.gap);
        self.ear.place(&self.goal.spot, self.rate);
    }

    /// Empties the line and jumps to where `hull` is: for a ship that has just appeared.
    pub fn clear(&mut self, hull: &HullSound, absorb: &Absorb) {
        self.line.fill(0.0);
        self.jump(hull, absorb);
        self.dull = Lag::default();
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
        let was = self.delay[0].0;
        let first = self.delay[0].to(self.goal.delay, self.k);
        let slope = self.slope.0.to(first - was, self.slope.1);
        let then = first / (1.0 + slope.clamp(-MAX_PACE, MAX_PACE));
        let before = self.delay[1].0;
        let wanted = self.delay[1].to(then.min(self.most()), self.k);
        let delay = wanted.clamp(before - MAX_PACE, before + MAX_PACE).max(1.0);
        self.delay[1].0 = delay;
        let gap = self.gap.to(self.goal.gap, self.k);
        let late = (delay + gap).min(self.most());
        let heard = self.tap(delay) + BOUNCE.2 * self.tap(late);
        let soft = self.dull.to(heard, self.goal.dull);
        self.ear.frame(soft, 0.0)
    }
}
