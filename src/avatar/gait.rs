//! The leg cycle: how long a stride is, where each foot is in it, and the leg that reaches it.
//!
//! A phase runs from 0 to 1 over one stride of two steps and advances with the distance walked.
//! A foot is planted for the first [`duty`] share of its own phase and moves back under the body
//! by exactly the distance walked, so it does not slide. The right foot runs half a phase late.

use std::f32::consts::PI;

use super::clock::ramp;
use super::tune::{BOB, REACH, RUN_SPAN, SHIN, STEP_LIFT, STRIDE, STRIDE_SPAN, THIGH};

/// Where an ankle is relative to its place when standing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Foot {
    /// How far ahead of the hip it is, in meters.
    pub ahead: f32,
    /// How high it is off the ground, in meters.
    pub lift: f32,
    /// Whether it carries the body: a planted foot must not slide.
    pub planted: bool,
}

/// Returns the length of a stride in meters at `speed`: longer when faster.
pub fn stride_length(speed: f32) -> f32 {
    (STRIDE.0 + STRIDE.1 * speed.abs()).clamp(STRIDE_SPAN.0, STRIDE_SPAN.1)
}

/// Returns the share of a stride each foot spends planted at `speed`, at most a half.
///
/// The cadence in strides per second is `speed / stride_length(speed)`: 2.5 at a walk of 5 m/s
/// and 3 at a sprint of 8.5 m/s.
pub fn duty(speed: f32) -> f32 {
    (REACH / stride_length(speed)).min(0.5)
}

/// Returns the phase after walking `moved` meters at `speed`.
pub fn advance(phase: f32, moved: f32, speed: f32) -> f32 {
    (phase + moved / stride_length(speed)).rem_euclid(1.0)
}

/// Returns how much of a run the stride is at `speed`, from 0 (walk) to 1 (sprint).
pub fn run(speed: f32) -> f32 {
    ramp(RUN_SPAN.0, RUN_SPAN.1, speed.abs())
}

/// Returns where the foot whose own phase is `phase` is, at `speed`.
pub fn foot(phase: f32, speed: f32) -> Foot {
    let (share, at) = (duty(speed), phase.rem_euclid(1.0));
    let travel = share * stride_length(speed);
    if at < share {
        let ahead = travel * (0.5 - at / share);
        return Foot {
            ahead,
            lift: 0.0,
            planted: true,
        };
    }
    let swing = (at - share) / (1.0 - share);
    let height = STEP_LIFT.0 + (STEP_LIFT.1 - STEP_LIFT.0) * run(speed);
    Foot {
        ahead: -0.5 * travel * (PI * swing).cos(),
        lift: height * (PI * swing).sin(),
        planted: false,
    }
}

/// Returns the height of the body bob at `phase`: lowest when a foot is under the body, highest in flight.
pub fn bob(phase: f32, speed: f32) -> f32 {
    let height = BOB.0 + BOB.1 * speed.abs();
    -height * (4.0 * PI * (phase - duty(speed) * 0.5)).cos()
}

/// Returns the hip swing and the knee bend that put the ankle `ahead` of and `down` from the hip.
///
/// The knee always points forward: its bend is never negative. A target out of reach is aimed at
/// with a nearly straight leg.
pub fn leg_ik(ahead: f32, down: f32) -> (f32, f32) {
    let reach = ahead.hypot(down).clamp(0.05, THIGH + SHIN - 1e-4);
    let inner = (THIGH * THIGH + SHIN * SHIN - reach * reach) / (2.0 * THIGH * SHIN);
    let lead = (THIGH * THIGH + reach * reach - SHIN * SHIN) / (2.0 * THIGH * reach);
    let knee = PI - inner.clamp(-1.0, 1.0).acos();
    (ahead.atan2(down) + lead.clamp(-1.0, 1.0).acos(), knee)
}
