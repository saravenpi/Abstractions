//! The warp as one sequence. The drive of the ship goes in, one state comes out, and every warp
//! visual is a pure function of that state: how far the sky crowds ahead, how long the stars
//! streak, the light ahead, the flash of the drop-out, the shake and the engine glow.
//!
//! The state moves at bounded rates in seconds, never per frame, so a slow frame or a fast
//! clock gives the same sequence, and a warp cut short falls back from wherever it stands.

mod lens;
mod look;
mod optics;

#[cfg(test)]
mod shader;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_sequence;

pub use lens::Lens;
pub use optics::CARRIED;

use bevy::prelude::*;

use super::entry::FlightFx;

/// Seconds the charge of the sequence needs at least to rise from nothing to full.
const CHARGE_TIME: f32 = 1.0;

/// Seconds the charge needs to fall from full to nothing: the spool in reverse, quicker.
pub const DROP_TIME: f32 = 1.6;

/// Seconds the velocity needs at least to rise from rest to the top speed.
const ENGAGE_TIME: f32 = 3.0;

/// Seconds the velocity needs to fall from the top speed to rest, shorter than [`DROP_TIME`].
const STOP_TIME: f32 = 1.2;

/// Velocity of a drive that has just engaged, as a share of the top speed: engaging is the
/// first step of the way, whatever the speed of the ship.
pub const ENGAGED: f32 = 0.25;

/// What the warp drive of the ship is doing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Drive {
    /// How far the drive is on: 0 when off, below 1 while it spools, 1 once it is engaged.
    pub charge: f32,
    /// Speed of an engaged drive as a share of its top speed, from 0 to 1.
    pub speed: f32,
}

/// The stage of a warp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// No warp.
    #[default]
    Idle,
    /// The drive charges.
    Spool,
    /// The drive has engaged and the ship gathers speed.
    Engage,
    /// The ship flies at the speed the drive gives it.
    Cruise,
    /// The drive is off and the sequence falls back to rest.
    DropOut,
}

/// The state of the warp sequence.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Sequence {
    /// The stage of the warp.
    pub phase: Phase,
    /// How far the stage has gone, from 0 to 1.
    pub progress: f32,
    /// How far the sequence is charged, from 0 to 1: it follows the charge of the drive.
    pub charge: f32,
    /// Velocity as a share of the top warp speed, from 0 to 1.
    pub velocity: f32,
}

/// Returns `value` moved toward `target` by at most `up` when it rises and `down` when it falls.
fn toward(value: f32, target: f32, up: f32, down: f32) -> f32 {
    value + (target - value).clamp(-down, up)
}

impl Drive {
    /// Returns the charge and the velocity the sequence heads for.
    fn wanted(self) -> (f32, f32) {
        let charge = self.charge.clamp(0.0, 1.0);
        if charge < 1.0 {
            return (charge, 0.0);
        }
        (1.0, ENGAGED + (1.0 - ENGAGED) * self.speed.clamp(0.0, 1.0))
    }
}

impl Sequence {
    /// Returns the state `dt` seconds later under `drive`.
    ///
    /// The charge and the velocity follow the drive at bounded rates, falling faster than
    /// they rise, so nothing ever steps: a drive that stops, at any moment, starts the
    /// drop-out from the state it is in.
    pub fn advance(self, drive: Drive, dt: f32) -> Self {
        let dt = dt.max(0.0);
        let (charge_to, velocity_to) = drive.wanted();
        let charge = toward(self.charge, charge_to, dt / CHARGE_TIME, dt / DROP_TIME);
        let velocity = toward(self.velocity, velocity_to, dt / ENGAGE_TIME, dt / STOP_TIME);
        let next = Self {
            charge,
            velocity,
            ..self
        };
        next.staged(charge_to, velocity_to, dt)
    }

    /// Returns this state with the stage and the progress that its charge, its velocity and
    /// the values the drive heads for make, `dt` seconds after the stage it holds.
    ///
    /// The drop-out counts its own time from wherever it starts, and ends when nothing is left.
    fn staged(self, charge_to: f32, velocity_to: f32, dt: f32) -> Self {
        let level = self.charge.max(self.velocity);
        let cruising = self.phase == Phase::Cruise || self.velocity >= velocity_to;
        let dropping = self.phase == Phase::DropOut;
        let (phase, progress) = match charge_to {
            wanted if wanted <= 0.0 && level <= 0.0 => (Phase::Idle, 0.0),
            wanted if wanted <= 0.0 && dropping => (Phase::DropOut, self.progress + dt / DROP_TIME),
            wanted if wanted <= 0.0 => (Phase::DropOut, 0.0),
            wanted if wanted < 1.0 => (Phase::Spool, self.charge),
            _ if cruising => (Phase::Cruise, 1.0),
            _ => (Phase::Engage, self.velocity / velocity_to),
        };
        Self {
            phase,
            progress: progress.min(1.0),
            ..self
        }
    }
}

/// Moves the sequence on with the drive the ship reports and derives what the sky draws from it.
/// The heading is the nose of the ship; without one the last heading stays.
pub(super) fn run(
    time: Res<Time>,
    flight: Res<FlightFx>,
    mut sequence: ResMut<Sequence>,
    mut lens: ResMut<Lens>,
) {
    let drive = Drive {
        charge: flight.warp,
        speed: flight.pace,
    };
    let next = sequence.advance(drive, time.delta_secs());
    sequence.set_if_neq(next);
    let heading = flight.nose.try_normalize().unwrap_or(lens.heading);
    lens.set_if_neq(next.lens(heading));
}

/// Registers the sequence and runs it once the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Sequence>()
        .init_resource::<Lens>()
        .add_systems(PostUpdate, run.after(super::sun::track));
}
