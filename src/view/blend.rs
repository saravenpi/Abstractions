//! The two blends of the rig: between first and third person, and between the ground and flight.

use super::io::Kind;
use super::mode::{ViewMode, ViewPrefs};
use super::smooth::toward;
use super::tune::{FLIGHT_TIME, SWITCH_TIME};
use crate::render::smoothstep;

/// How far each blend has run. Both run at a steady pace and are eased when read.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) struct Blend {
    /// From first person, 0, to third person, 1.
    person: f32,
    /// From the ground rig, 0, to the flight rig, 1.
    flight: f32,
}

/// Returns where the two blends are heading for a target of `kind`.
fn goals(kind: Kind, prefs: &ViewPrefs) -> (f32, f32) {
    let third = prefs.mode(kind) == ViewMode::Third;
    (
        f32::from(u8::from(third)),
        f32::from(u8::from(kind == Kind::Flying)),
    )
}

impl Blend {
    /// Returns the blends at rest for a target of `kind`.
    pub(super) fn settled(kind: Kind, prefs: &ViewPrefs) -> Self {
        let (person, flight) = goals(kind, prefs);
        Self { person, flight }
    }

    /// Runs both blends for `dt` seconds.
    pub(super) fn step(&mut self, kind: Kind, prefs: &ViewPrefs, dt: f32) {
        let (person, flight) = goals(kind, prefs);
        self.person = toward(self.person, person, dt / SWITCH_TIME);
        self.flight = toward(self.flight, flight, dt / FLIGHT_TIME);
    }

    /// Returns how much of a third person view to show, eased at both ends.
    pub(super) fn person(&self) -> f32 {
        smoothstep(0.0, 1.0, self.person)
    }

    /// Returns how much of the flight rig to show, eased at both ends.
    pub(super) fn flight(&self) -> f32 {
        smoothstep(0.0, 1.0, self.flight)
    }
}
