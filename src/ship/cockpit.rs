//! The cockpit as the pilot uses it: where the controls and the instruments are in the hull,
//! how far each one is moved by the real state of the ship, and the rules that say so.
//!
//! Everything here is plain data in the axes of the hull: the nose along -Z, the roof along
//! +Y. The meshes are built around these places, the parts are moved by these rules and the
//! hands of the pilot hold the grips these rules give.

mod parts;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_app;

use bevy::prelude::*;

use super::cruise::top_speed;
use super::telemetry::{ShipTelemetry, WarpState};
use super::warp::fraction;

pub use parts::{Kit, fit};

/// Height of the floor of the tub.
pub const FLOOR: f32 = -0.34;

/// Height of the top of the seat pan: the pilot sits on it.
pub const PAN: f32 = 0.06;

/// Where the front of the backrest and of the headrest is along z: the back of the pilot.
pub const BACK: f32 = -0.975;

/// Where the centre stick hinges, on its column between the knees of the pilot.
pub const STICK: Vec3 = Vec3::new(0.0, 0.1, -1.42);

/// Length of the stick from its hinge to the top of its grip, and how far under the top the
/// hand holds it.
pub const STICK_LENGTH: (f32, f32) = (0.34, 0.04);

/// Tilt of the stick at full pitch or roll, and its twist at full yaw, in radians.
pub const STICK_REACH: (f32, f32) = (0.18, 0.3);

/// Where the throttle lever hinges, on top of the left console.
pub const LEVER: Vec3 = Vec3::new(-0.27, 0.22, -1.4);

/// Length of the throttle lever from its hinge to the top of its handle, and how far under the
/// top the hand holds it.
pub const LEVER_LENGTH: (f32, f32) = (0.2, 0.025);

/// Tilt of the lever from upright, in radians: back by that much at idle, forward at full.
pub const LEVER_SWING: f32 = 0.35;

/// Rate per second at which the controls follow what is asked of them, so that a key that
/// goes down in one frame moves the stick over a few.
pub const FOLLOW: f32 = 14.0;

/// Rate per second at which the instruments follow what they measure.
pub const SETTLE: f32 = 8.0;

/// Where the aft face of the instrument pods is along z: the plane the instruments are on.
pub const PANEL_Z: f32 = -1.95;

/// Where the controls of a ship are: what its pilot asks of it, or where its parts stand.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Levers {
    /// The stick, from -1 to 1 around the body axes: pitch up, yaw left, roll left.
    pub stick: Vec3,
    /// The throttle lever, from 0 (idle) to 1.
    pub throttle: f32,
}

impl Levers {
    /// Returns the controls `dt` seconds later, on their way to `asked`.
    pub fn eased(self, asked: Levers, dt: f32) -> Levers {
        let share = 1.0 - (-FOLLOW * dt).exp();
        Levers {
            stick: self.stick + (asked.stick - self.stick) * share,
            throttle: self.throttle + (asked.throttle - self.throttle) * share,
        }
    }

    /// Returns the turn of the stick in its hinge: pulled back for pitch up, tilted left for
    /// a roll to the left, twisted left for a yaw to the left.
    pub fn stick_turn(&self) -> Quat {
        let held = self.stick.clamp(Vec3::NEG_ONE, Vec3::ONE);
        Quat::from_rotation_x(held.x * STICK_REACH.0)
            * Quat::from_rotation_z(held.z * STICK_REACH.0)
            * Quat::from_rotation_y(held.y * STICK_REACH.1)
    }

    /// Returns the turn of the throttle lever in its hinge: back at idle, forward at full.
    pub fn lever_turn(&self) -> Quat {
        Quat::from_rotation_x(LEVER_SWING * (1.0 - 2.0 * self.throttle.clamp(0.0, 1.0)))
    }

    /// Returns where the hands of the pilot hold the controls, in hull axes: the left hand on
    /// the throttle, the right hand on the stick.
    pub fn grips(&self) -> [Vec3; 2] {
        let lever = Vec3::Y * (LEVER_LENGTH.0 - LEVER_LENGTH.1);
        let stick = Vec3::Y * (STICK_LENGTH.0 - STICK_LENGTH.1);
        [
            LEVER + self.lever_turn() * lever,
            STICK + self.stick_turn() * stick,
        ]
    }
}

/// An instrument of the dashboard. Each one shows one real number of the ship, in white on black.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dial {
    /// A bar: the speed against the top speed the lever can ask for here, or in warp the
    /// warp speed against the fastest jump.
    Speed,
    /// A bar: how far the warp drive has spooled, full while the drive is engaged.
    Charge,
    /// A lamp: how far the landing gear is down.
    Gear,
}

impl Dial {
    /// Every instrument.
    pub const ALL: [Dial; 3] = [Dial::Speed, Dial::Charge, Dial::Gear];

    /// Returns what the instrument reads, from 0 to 1, from the instruments of the ship and
    /// the extension of its `gear`.
    pub fn reading(self, telemetry: &ShipTelemetry, gear: f64) -> f32 {
        let value = match (self, telemetry.warp) {
            (Dial::Speed, WarpState::Active(speed)) => fraction(speed),
            (Dial::Speed, _) => telemetry.surface_speed / top_speed(telemetry.altitude_ground),
            (Dial::Charge, WarpState::Off) => 0.0,
            (Dial::Charge, WarpState::Spooling(ready)) => ready,
            (Dial::Charge, WarpState::Active(_)) => 1.0,
            (Dial::Gear, _) => gear,
        };
        if value.is_finite() {
            value.clamp(0.0, 1.0) as f32
        } else {
            0.0
        }
    }

    /// Returns where the lit part of the instrument is on the panel when it reads `value`: a
    /// unit bar that starts at its left end, stretched to the reading.
    pub fn place(self, value: f32) -> Transform {
        let lit = value.clamp(0.0, 1.0).max(1.0e-3);
        let (from, size) = match self {
            Dial::Speed => (Vec2::new(-0.28, 0.527), Vec2::new(0.14 * lit, 0.045)),
            Dial::Charge => (Vec2::new(0.14, 0.548), Vec2::new(0.14 * lit, 0.03)),
            Dial::Gear => (Vec2::new(0.21 - 0.02 * lit, 0.497), Vec2::splat(0.04 * lit)),
        };
        Transform::from_translation(from.extend(PANEL_Z + 0.006)).with_scale(size.extend(1.0))
    }
}

/// The cockpit of one ship, a component of the ship entity. Whoever flies the ship writes
/// what is `asked` and what the instruments `read`; the parts of that ship are moved from what
/// is `shown`, which follows them without a step. Every ship on screen has its own.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Cockpit {
    /// Where the pilot holds the stick and the lever right now.
    pub asked: Levers,
    /// What each instrument measures right now, from 0 to 1, in the order of [`Dial::ALL`].
    pub read: [f32; 3],
    /// Where the stick and the lever are drawn, on their way to what is asked.
    pub shown: Levers,
    /// What each instrument shows, on its way to what it reads.
    pub gauges: [f32; 3],
    /// The player entity that sits in the seat, when one does.
    pub pilot: Option<Entity>,
}

impl Cockpit {
    /// Moves what is shown toward what is asked and read, for `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        let share = 1.0 - (-SETTLE * dt).exp();
        self.shown = self.shown.eased(self.asked, dt);
        for (shown, read) in self.gauges.iter_mut().zip(self.read) {
            *shown += (read - *shown) * share;
        }
    }
}

/// Registers the systems of the cockpit: the local ship is fed from its helm, then the parts
/// of every cockpit are moved.
pub(super) fn plugin(app: &mut App) {
    let run = (parts::feed, parts::work).chain();
    app.add_systems(Update, run.in_set(super::ShipSet::Place));
}
