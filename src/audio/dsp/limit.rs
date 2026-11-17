//! The output guard: no constant offset, linear below the knee, a soft curve above it, never past
//! full scale.

/// Level below which the signal passes untouched.
pub const KNEE: f32 = 0.6;

/// Highest level the limiter can output.
pub const CEILING: f32 = 0.98;

/// Returns `x` bounded to [`CEILING`], bending smoothly above [`KNEE`]. A non-finite input gives silence.
pub fn limit(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    let level = x.abs();
    if level <= KNEE {
        return x;
    }
    let room = CEILING - KNEE;
    (KNEE + room * ((level - KNEE) / room).tanh()).min(CEILING) * x.signum()
}

/// Removes any constant offset with a high-pass far below hearing.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DcBlock {
    /// Previous input.
    x: f32,
    /// Previous output.
    y: f32,
}

impl DcBlock {
    /// Runs one sample with the pole `r`, just under 1.
    pub fn run(&mut self, x: f32, r: f32) -> f32 {
        let y = x - self.x + r * self.y;
        self.x = x;
        self.y = if y.abs() < 1e-12 { 0.0 } else { y };
        self.y
    }
}
