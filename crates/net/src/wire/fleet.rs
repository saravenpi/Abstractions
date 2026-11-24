//! Shared ship messages: who pilots which player's ship and requests to board or leave it.

use serde::{Deserialize, Serialize};

use crate::log::PlayerId;

/// One piloted ship and the player currently holding its controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipPilot {
    /// Player id that owns the ship.
    pub ship: PlayerId,
    /// Player id piloting the ship.
    pub pilot: PlayerId,
}

/// What a player asks the server when boarding or leaving a ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipAct {
    /// Requests to take the pilot seat of `PlayerId`'s ship.
    Board(PlayerId),
    /// Releases the pilot seat of `PlayerId`'s ship after landing.
    Leave(PlayerId),
}

/// What the server tells players about ship pilots and authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipEvent {
    /// Active ship pilots in the room when joining.
    World(Vec<ShipPilot>),
    /// A player took the controls of a ship.
    Boarded {
        /// Ship owner id.
        ship: PlayerId,
        /// Pilot player id.
        pilot: PlayerId,
    },
    /// A player left the controls of a ship.
    Left {
        /// Ship owner id.
        ship: PlayerId,
        /// Pilot player id.
        pilot: PlayerId,
    },
    /// Boarding a ship was refused because someone else already pilots it.
    Denied(PlayerId),
    /// A disconnected owner's unoccupied ship left the session.
    Gone(PlayerId),
}
