//! Framerate independent smoothing: every function takes the frame time and gives the same curve at any rate.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

/// Returns the share of a gap that closes in `dt` seconds at `rate` per second.
pub(super) fn closed(rate: f32, dt: f32) -> f32 {
    1.0 - (-rate * dt).exp()
}

/// Returns `angle` wrapped to the half turn either side of zero.
pub(super) fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Returns `value` moved toward `goal` by at most `step`.
pub(super) fn toward(value: f32, goal: f32, step: f32) -> f32 {
    value + (goal - value).clamp(-step, step)
}

/// Returns how far a follower trails a value that moved by `step` during `dt` seconds, given the
/// `lag` it trailed by before. The follower closes the gap at `rate`.
///
/// This is the exact answer for a value that moves steadily during the frame, so a steady turn
/// is trailed by the same angle at every frame rate.
pub(super) fn trail(lag: Vec2, step: Vec2, rate: f32, dt: f32) -> Vec2 {
    let steady = step / (rate * dt);
    steady + (lag - steady) * (-rate * dt).exp()
}

/// A critically damped spring: it reaches its goal as fast as a spring can without swinging past it.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) struct Spring {
    /// Where the spring is.
    pub value: Vec3,
    /// How fast it moves.
    pub velocity: Vec3,
}

impl Spring {
    /// Moves the spring toward `goal` for `dt` seconds at stiffness `omega`, with the exact
    /// solution of its motion.
    pub(super) fn step(&mut self, goal: Vec3, omega: f32, dt: f32) {
        let gap = self.value - goal;
        let pull = (self.velocity + gap * omega) * dt;
        let decay = (-omega * dt).exp();
        self.value = goal + (gap + pull) * decay;
        self.velocity = (self.velocity - pull * omega) * decay;
    }
}
