//! Filters: a one-pole smoother and a two-pole state variable filter.

use std::f32::consts::{PI, TAU};

/// Returns the one-pole coefficient that follows a target with a corner at `hz`.
pub fn pole(hz: f32, rate: f32) -> f32 {
    1.0 - (-TAU * hz / rate).exp()
}

/// Returns the one-pole coefficient that covers 63 percent of a step in `seconds`.
pub fn glide(seconds: f32, rate: f32) -> f32 {
    1.0 - (-1.0 / (seconds * rate)).exp()
}

/// Distance under which a settling value jumps onto its target: a gain step of 0.01 dB.
///
/// It must stay above 2e-4, where 32-bit rounding stops a slow lag just short of 1.
pub const SETTLED: f32 = 1e-3;

/// A one-pole low-pass, also used to smooth every control value so that no gain ever steps.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Lag(pub f32);

impl Lag {
    /// Moves toward `target` by the fraction `k` and returns the new value.
    pub fn to(&mut self, target: f32, k: f32) -> f32 {
        self.0 += k * (target - self.0);
        self.0
    }

    /// Moves toward `target` like [`Lag::to`], then lands on it once closer than [`SETTLED`].
    ///
    /// A value that must reach exactly 0 or 1, such as a gain that silences a voice, uses this.
    pub fn settle(&mut self, target: f32, k: f32) -> f32 {
        let next = self.to(target, k);
        if (target - next).abs() < SETTLED {
            self.0 = target;
        }
        self.0
    }
}

/// A two-pole state variable filter that stays stable at any cutoff.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Svf {
    /// Coefficients set by [`Svf::tune`].
    a: [f32; 3],
    /// Damping, the inverse of the resonance.
    k: f32,
    /// The two integrator states.
    s: [f32; 2],
}

impl Svf {
    /// Sets the cutoff in hertz and the resonance `q` for a sample rate.
    pub fn tune(&mut self, hz: f32, q: f32, rate: f32) {
        let g = (PI * (hz / rate).clamp(0.0002, 0.45)).tan();
        self.k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a = [a1, g * a1, g * g * a1];
    }

    /// Runs one sample and returns the low-pass and band-pass outputs.
    pub fn run(&mut self, x: f32) -> (f32, f32) {
        let v3 = x - self.s[1];
        let v1 = self.a[0] * self.s[0] + self.a[1] * v3;
        let v2 = self.s[1] + self.a[1] * self.s[0] + self.a[2] * v3;
        self.s = [2.0 * v1 - self.s[0], 2.0 * v2 - self.s[1]];
        (v2, v1 * self.k)
    }

    /// Runs one sample and returns the low-pass output.
    pub fn low(&mut self, x: f32) -> f32 {
        self.run(x).0
    }

    /// Runs one sample and returns the band-pass output, unity gain at the cutoff.
    pub fn band(&mut self, x: f32) -> f32 {
        self.run(x).1
    }
}
