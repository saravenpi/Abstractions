//! Tests of the flight gear: pure runs of the model, no window.

mod air;
mod assist;
mod boots;
mod feel;
mod fly;
mod glide;
mod ground;
mod jet;
mod loops;
mod measure;
mod pilot;
mod rate;
mod rolls;
mod spread;
mod stall;
mod transitions;
mod turn;
mod vacuum;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::{FlightState, GearInput, Mode, step};

/// Frame time of the test flights in seconds.
pub(super) const DT: f32 = 1.0 / 60.0;

/// A body flown by the model alone: the state and where it has got to.
#[derive(Debug, Clone, Copy)]
pub(super) struct Flight {
    /// The gear state.
    pub(super) state: FlightState,
    /// Position in metres, +Y up.
    pub(super) pos: DVec3,
    /// Largest change of velocity in one frame so far, in metres per second.
    pub(super) kick: f32,
    /// Largest change of attitude in one frame so far, in radians.
    pub(super) twist: f32,
}

impl Flight {
    /// Returns a body in `mode` moving at `velocity`, with the spool and wings already settled.
    pub(super) fn new(mode: Mode, velocity: Vec3) -> Self {
        let state = FlightState {
            mode,
            velocity,
            wings: f32::from(u8::from(mode.winged())),
            thrust: f32::from(u8::from(matches!(mode, Mode::Fly | Mode::BootFall))),
            ..FlightState::default()
        };
        Self {
            state,
            pos: DVec3::ZERO,
            kick: 0.0,
            twist: 0.0,
        }
    }

    /// Advances the body by `dt` seconds and moves it by the new velocity.
    pub(super) fn tick(&mut self, input: &GearInput, dt: f32) {
        let before = self.state;
        self.state = step(&self.state, input, dt);
        self.kick = self
            .kick
            .max((self.state.velocity - before.velocity).length());
        self.twist = self
            .twist
            .max(self.state.attitude.angle_between(before.attitude));
        self.pos += self.state.velocity.as_dvec3() * f64::from(dt);
    }

    /// Advances the body for `seconds` at the test frame time.
    pub(super) fn run(&mut self, input: &GearInput, seconds: f32) {
        for _ in 0..(seconds / DT).round() as u32 {
            self.tick(input, DT);
        }
    }

    /// Returns the mechanical energy per kilogram under `gravity`.
    pub(super) fn energy(&self, gravity: f32) -> f64 {
        0.5 * self.state.velocity.as_dvec3().length_squared() + f64::from(gravity) * self.pos.y
    }
}

/// Returns an input holding the wings open, the view steering them straight ahead
/// `elevation` radians above the horizon.
pub(super) fn gliding(elevation: f32) -> GearInput {
    GearInput {
        deploy: true,
        look: Vec3::new(0.0, elevation.sin(), -elevation.cos()),
        aim: Some(Vec2::new(0.0, elevation)),
        ..GearInput::default()
    }
}

/// Returns an input holding wings and boosters, the view steering them `elevation` radians
/// above the horizon.
pub(super) fn flying(elevation: f32) -> GearInput {
    GearInput {
        boost: true,
        ..gliding(elevation)
    }
}

/// Returns an input holding the boots alone.
pub(super) fn braking() -> GearInput {
    GearInput {
        boost: true,
        ..GearInput::default()
    }
}

/// Returns the unit look direction at `yaw` radians to the left and `elevation` radians up.
pub(super) fn looking(yaw: f32, elevation: f32) -> Vec3 {
    Quat::from_rotation_y(yaw) * Vec3::new(0.0, elevation.sin(), -elevation.cos())
}
