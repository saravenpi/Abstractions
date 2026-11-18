//! Tests of the body. The shared helpers build the states and instants the poses are sampled at.

mod app;
mod blending;
mod body;
mod diving;
mod dressing;
mod face;
mod filling;
mod flown;
mod gait;
mod loops;
mod neck;
mod poses;
mod wings;

use bevy::prelude::*;

use super::clock::Clock;
use super::pose::Pose;
use super::tune::CROUCH;
use super::{AvatarPose, Mode};

/// Seconds in a frame at sixty frames per second.
pub const FRAME: f32 = 1.0 / 60.0;

/// Returns a state in `mode` with every other field at the strength `k`, from 0 to 1.
pub fn scene(mode: Mode, k: f32) -> AvatarPose {
    AvatarPose {
        mode,
        speed: 8.5 * k,
        vertical_speed: -30.0 * k + 6.0,
        stride_distance: 0.0,
        bank: 1.4 * (k - 0.5),
        pitch: 1.2 * (0.5 - k),
        thrust: k,
        wings: k,
        airspeed: 60.0 * k,
        look: Vec2::new(0.8 * (k - 0.5), 0.6 * (0.5 - k)),
        landed_hard: k,
    }
}

/// Returns instants spread over time, the leg cycle, the swim cycle and the landing crouch.
pub fn clocks() -> Vec<Clock> {
    (0..48)
        .map(|i| {
            let t = i as f32;
            Clock {
                time: f64::from(t) * 1.37,
                phase: (t * 0.0213 + t / 48.0).fract(),
                stroke: t * 0.27,
                crouch: CROUCH.1 * (t * 0.61).sin().max(0.0),
                still: t * 2.0,
                ..Clock::default()
            }
        })
        .collect()
}

/// Returns every state and instant the poses of a mode are sampled at.
pub fn samples(mode: Mode) -> Vec<(AvatarPose, Clock)> {
    [0.0, 0.2, 0.5, 0.8, 1.0]
        .into_iter()
        .flat_map(|k| {
            clocks()
                .into_iter()
                .map(move |clock| (scene(mode, k), clock))
        })
        .collect()
}

/// Returns the largest difference between two poses over every joint, in radians or meters.
pub fn gap(a: &Pose, b: &Pose) -> f32 {
    let (a, b) = (a.values(), b.values());
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}
