//! The shape of an exhaust from what drives it, as pure functions: its length, its width, its
//! brightness and the shock diamonds in it come from the thrust, the exit of the nozzle and the
//! pressure of the air around it. Thick air holds the gas in a narrow bright jet; thin air and
//! vacuum let it expand into a wide, faint plume.

use super::tune::{
    EXHAUST_SPEED, FAINT, LENGTH, OFF, SCROLL, SHOCK, SPOOL_DOWN, SPOOL_UP, WARP_LENGTH, WIDTH,
};

/// What a plume looks like right now.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Shape {
    /// Length in metres.
    pub length: f32,
    /// Radius in metres where the plume leaves the nozzle, from which it swells and narrows.
    pub width: f32,
    /// Brightness, 1 for a main engine at full thrust in thick air.
    pub bright: f32,
    /// How visible the shock diamonds are, from 0 to 1.
    pub shock: f32,
    /// Plume lengths per second the noise scrolls along the axis.
    pub scroll: f32,
}

/// Returns `t` eased in with the Hermite curve between `from` and `to`.
fn smooth(from: f32, to: f32, t: f32) -> f32 {
    let x = ((t - from) / (to - from)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Returns `now` moved toward `target` over `dt` seconds: up in `up` seconds, down in `down`.
pub fn ease(now: f32, target: f32, (up, down): (f32, f32), dt: f32) -> f32 {
    let rate = if target > now { up } else { down };
    now + (target - now) * (1.0 - (-dt / rate.max(1.0e-3)).exp())
}

/// Returns the spool of a main engine `dt` seconds on, from `now` toward the `thrust` asked.
pub fn spool(now: f32, thrust: f32, dt: f32) -> f32 {
    ease(now, thrust, (SPOOL_UP, SPOOL_DOWN), dt)
}

/// Returns the shape of the plume of a nozzle with an exit `exit` metres in radius at `thrust`,
/// 1 at full throttle and more with the boost, in air `air` times as thick as at sea level at
/// home. Nothing is drawn under [`OFF`].
pub fn shape(exit: f32, thrust: f32, air: f32) -> Shape {
    if thrust < OFF {
        return Shape::default();
    }
    let air = air.clamp(0.0, 1.0);
    let (vacuum, push) = (1.0 - air, thrust.sqrt());
    let length = 2.0 * exit * push * (LENGTH.0 + (LENGTH.1 - LENGTH.0) * vacuum);
    Shape {
        length,
        width: exit * push * (WIDTH.0 + (WIDTH.1 - WIDTH.0) * vacuum),
        bright: thrust * (1.0 - (1.0 - FAINT) * vacuum),
        shock: smooth(SHOCK.0, SHOCK.1, air) * thrust.min(1.0),
        scroll: (EXHAUST_SPEED / length).min(SCROLL),
    }
}

/// Returns the shape of the plume of the warp drive of a nozzle with an exit `exit` metres in
/// radius, `drive` up from 0 to 1: a long, narrow, bright streak, whatever the air.
pub fn warp(exit: f32, drive: f32) -> Shape {
    if drive < OFF {
        return Shape::default();
    }
    let length = 2.0 * exit * WARP_LENGTH * drive;
    Shape {
        length,
        width: exit * (1.0 - 0.45 * drive),
        bright: 1.4 * drive,
        shock: 0.0,
        scroll: SCROLL,
    }
}
