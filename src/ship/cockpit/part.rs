//! The moving parts of a cockpit, and where each one stands for the state of its cockpit.

use bevy::prelude::*;

use super::readings::Lamp;
use super::state::Cockpit;

/// A part of the cockpit that moves, a child of the ship that carries it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// The centre stick.
    Stick,
    /// The throttle lever.
    Lever,
    /// The gear lever.
    Gear,
    /// The warp switch.
    Switch,
    /// A rudder pedal, 0 for the left and 1 for the right.
    Pedal(usize),
    /// The loose strap.
    Strap,
    /// A lamp.
    Lamp(Lamp),
}

impl Part {
    /// Returns where the part stands in `cockpit` when it stands at `rest` with nothing asked
    /// of it. The levers turn in their hinges, the lamps are placed outright.
    pub fn placed(self, cockpit: &Cockpit, rest: Transform) -> Transform {
        let shown = &cockpit.shown;
        match self {
            Part::Stick => rest.with_rotation(shown.stick_turn()),
            Part::Lever => rest.with_rotation(shown.lever_turn()),
            Part::Gear => rest.with_rotation(shown.gear_turn()),
            Part::Switch => rest.with_rotation(shown.switch_turn()),
            Part::Pedal(side) => rest.with_rotation(shown.pedal_turn(side)),
            Part::Strap => rest.with_rotation(cockpit.strap.turn()),
            Part::Lamp(lamp) => lamp.place(cockpit.read.lamps[lamp as usize]),
        }
    }
}
