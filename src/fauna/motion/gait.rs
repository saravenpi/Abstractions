//! The gait clock: Froude number, gait mix, duty factor, stride length and the phase they drive.
//!
//! The gait is chosen by the Froude number `Fr = v^2 / (g * leg)`, with the planet's own gravity,
//! after Alexander, "The gaits of bipedal and quadrupedal animals" (1984,
//! https://cob.silverchair.com/jeb/article-pdf/115/1/263/2432986/jexbio_115_1_263.pdf): a walk under
//! about 0.5, a trot up to about 2.5, then a gallop. The duty factor falls with speed and the
//! relative stride length is `2.3 * Fr^0.3`. Gaits blend smoothly across each change. The phase runs
//! from 0 to 1 over one stride and advances with the distance walked, never with time.

use super::lag;

/// How much of the trot and of the gallop is in the gait, each 0 to 1 over the one before.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mix {
    /// Share of trot over walk.
    pub trot: f32,
    /// Share of gallop over the walk and trot blend.
    pub gallop: f32,
}

/// Hermite ramp from 0 at `low` to 1 at `high`.
pub fn ramp(low: f32, high: f32, x: f32) -> f32 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the Froude number of `speed` in metres per second on legs `leg` metres long under `gravity`.
pub fn froude(speed: f32, gravity: f32, leg: f32) -> f32 {
    speed * speed / (gravity.max(0.1) * leg.max(0.01))
}

impl Mix {
    /// Returns the gait mix at Froude number `fr`.
    pub fn of(fr: f32) -> Self {
        Self {
            trot: ramp(0.4, 0.65, fr),
            gallop: ramp(2.2, 2.9, fr),
        }
    }

    /// Returns the share of a stride each foot spends on the ground at Froude number `fr`.
    pub fn duty(&self, fr: f32) -> f32 {
        let walk = (0.72 - 0.3 * fr).max(0.56);
        let trot = (0.48 - 0.05 * (fr - 0.5)).clamp(0.34, 0.48);
        let gallop = (0.36 - 0.02 * (fr - 2.5)).clamp(0.24, 0.36);
        let steady = walk + (trot - walk) * self.trot;
        steady + (gallop - steady) * self.gallop
    }
}

/// Returns the stride length in metres of legs `leg` metres long at Froude number `fr`.
pub fn stride(fr: f32, leg: f32) -> f32 {
    leg * (2.3 * fr.max(0.0).powf(0.3)).max(0.6)
}

/// The phase of the stride and the gait it runs.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Clock {
    /// Phase of the stride, 0 to 1.
    pub phase: f32,
    /// Gait mix.
    pub mix: Mix,
    /// Share of a stride each foot is planted.
    pub duty: f32,
    /// Stride length in metres.
    pub stride: f32,
}

impl Clock {
    /// Advances the clock by `moved` metres walked at `speed` on legs `leg` long under `gravity`.
    pub fn advance(&mut self, moved: f32, speed: f32, gravity: f32, leg: f32) {
        let fr = froude(speed, gravity, leg);
        self.mix = Mix::of(fr);
        self.duty = self.mix.duty(fr);
        self.stride = stride(fr, leg);
        self.phase = (self.phase + moved / self.stride).rem_euclid(1.0);
    }

    /// Returns the own phase of a foot that lags the clock by `lag`, 0 to 1: planted below the duty.
    pub fn own(&self, lag: f32) -> f32 {
        (self.phase - lag).rem_euclid(1.0)
    }

    /// Returns the lag of leg `rank` and `side` of a body with `legs` legs in `ranks` ranks.
    pub fn lag(&self, legs: usize, ranks: u8, rank: u8, side: i8) -> f32 {
        lag::blended(
            lag::Seat {
                legs,
                ranks,
                rank,
                side,
            },
            &self.mix,
        )
    }
}
