//! The two ears, on the audio thread: one sound in, a stereo frame out, for headphones and for
//! speakers alike.
//!
//! The way to the head is a low-pass at the corner of the spot. Then the outer ear: a gentle shelf
//! takes some of the highs of a sound from behind. The pan is equal power, the same level ahead
//! on both ears, leaning to the near ear as the sound goes to one side. The far ear hears it a
//! little later, by the time sound takes round the head, and through the shadow of the head, a
//! low-pass that closes as the sound goes to the side. What the body carries comes in after all
//! of it, the same in both ears. A voice costs about 25 floating point operations per sample.

use std::f32::consts::{FRAC_PI_4, SQRT_2};

use super::spot::Spot;
use super::tune::{GAP, HEAD, LEAN, REAR, SHADOW_HZ};
use crate::audio::dsp::filter::{Lag, pole};

/// Mask that wraps an index into the line to the far ear.
const MASK: usize = GAP - 1;

/// Returns the seconds the far ear hears a sound from `side` later than the near one, where
/// sound travels at `sound` metres per second: Woodworth's law for a round head.
pub fn around(side: f32, sound: f32) -> f32 {
    let angle = side.abs().min(1.0).asin();
    HEAD / sound * (angle + angle.sin())
}

/// What one sound becomes at the two ears.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ear {
    /// Gains of the air into the left and the right ear, smoothed.
    gains: [Lag; 2],
    /// What the gains move toward.
    goal: [f32; 2],
    /// Coefficient of the smoothing of the gains: 1 for a sound placed once.
    glide: f32,
    /// Gain of what the body carries, into both ears.
    body: f32,
    /// The low-pass of the way, and its coefficient.
    way: (Lag, f32),
    /// The shelf of the outer ear: its low part, its coefficient and the share of the highs kept.
    rear: (Lag, f32, f32),
    /// The shadow of the head on the far ear, and its coefficient.
    shadow: (Lag, f32),
    /// The sound on its way round the head to the far ear.
    line: [f32; GAP],
    /// Where the next sample is written.
    at: usize,
    /// Samples the far ear hears it later.
    late: f32,
    /// The far ear: 0 the left, 1 the right.
    far: usize,
    /// Share of the sound the cabin sends back as reflections.
    room: f32,
}

impl Ear {
    /// Builds an ear that hears nothing yet, whose gains glide by `glide` each sample: 1 for a
    /// sound placed once and never moved.
    pub const fn new(glide: f32) -> Self {
        Self {
            gains: [Lag(0.0); 2],
            goal: [0.0; 2],
            glide,
            body: 0.0,
            way: (Lag(0.0), 1.0),
            rear: (Lag(0.0), 1.0, 1.0),
            shadow: (Lag(0.0), 1.0),
            line: [0.0; GAP],
            at: 0,
            late: 0.0,
            far: 1,
            room: 0.0,
        }
    }

    /// Aims the ear at `spot` for a sample `rate`, keeping what is already on its way: for a
    /// source that moves.
    pub fn aim(&mut self, spot: &Spot, rate: f32) {
        let side = spot.side.clamp(-1.0, 1.0);
        let (right, left) = (FRAC_PI_4 * (1.0 + LEAN * side)).sin_cos();
        self.goal = [SQRT_2 * left * spot.air, SQRT_2 * right * spot.air];
        self.body = spot.body;
        self.room = spot.room;
        self.way.1 = pole(spot.corner, rate);
        let behind = (-spot.front).clamp(0.0, 1.0);
        self.rear = (self.rear.0, pole(REAR.0, rate), 1.0 - REAR.1 * behind);
        self.shadow.1 = pole(SHADOW_HZ / (side * side).max(1.0e-6), rate);
        self.late = (spot.gap * rate).clamp(0.0, (GAP - 2) as f32);
        self.far = usize::from(side <= 0.0);
    }

    /// Places the ear at `spot` at once, with nothing on its way: for a sound that starts now.
    pub fn place(&mut self, spot: &Spot, rate: f32) {
        *self = Self::new(self.glide);
        self.aim(spot, rate);
        self.gains = self.goal.map(Lag);
    }

    /// Returns the share of the sound the cabin sends back as reflections.
    pub fn room(&self) -> f32 {
        self.room
    }

    /// Returns the sample at the far ear now: the sound as it was `late` samples ago, through
    /// the shadow of the head.
    fn round(&mut self) -> f32 {
        let whole = self.late as usize;
        let part = self.late - whole as f32;
        let near = (self.at + GAP - whole) & MASK;
        let before = (near + GAP - 1) & MASK;
        let delayed = self.line[near] * (1.0 - part) + self.line[before] * part;
        self.shadow.0.to(delayed, self.shadow.1)
    }

    /// Renders one stereo frame of a sound whose share through the air is `air` and whose share
    /// through the body is `body`.
    pub fn frame(&mut self, air: f32, body: f32) -> [f32; 2] {
        let heard = self.way.0.to(air, self.way.1);
        let low = self.rear.0.to(heard, self.rear.1);
        let shelved = low + self.rear.2 * (heard - low);
        self.line[self.at] = shelved;
        let round = self.round();
        self.at = (self.at + 1) & MASK;
        let (glide, goal) = (self.glide, self.goal);
        let gains = [0, 1].map(|i| self.gains[i].to(goal[i], glide));
        let inner = body * self.body;
        let mut out = [shelved * gains[0] + inner, shelved * gains[1] + inner];
        out[self.far] = round * gains[self.far] + inner;
        out
    }
}
