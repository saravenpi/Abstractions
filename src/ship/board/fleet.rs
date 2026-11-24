//! Client-side ownership and boarding handshake state for shared multiplayer ships.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use crate::sim::PlayerId;
use crate::space::Frame;

/// Pose and mechanical state loaded into the local `Ship` when taking over a remote ship.
#[derive(Debug, Clone, PartialEq)]
pub struct SwapShip {
    /// Owner id of the boarded ship.
    pub ship: PlayerId,
    /// Coordinate frame of the ship.
    pub frame: Frame,
    /// Position in `frame`.
    pub pos: DVec3,
    /// Velocity in `frame`.
    pub vel: DVec3,
    /// Orientation in `frame`.
    pub rot: DQuat,
    /// Gear extension from 0 to 1.
    pub gear: f64,
    /// Accumulated hull damage from 0 to 1.
    pub damage: f64,
    /// True while resting landed.
    pub landed: bool,
    /// Headlight state.
    pub headlight: bool,
}

/// Server-granted boarding outcome applied by `board::board`.
#[derive(Debug, Clone, PartialEq)]
pub enum BoardGrant {
    /// Server confirmed taking the seat of the ship already loaded into the local `Ship`.
    Current(PlayerId),
    /// Server confirmed taking the seat of a remote ship loaded from `SwapShip`.
    Remote(Box<SwapShip>),
}

/// Tracks which ship this client currently simulates and pending online boarding actions.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct Fleet {
    /// True while a multiplayer `Link` is open.
    pub online: bool,
    /// Server player id initialized for this session.
    pub joined: Option<PlayerId>,
    /// Owner id of the ship loaded into the local `Ship`, or `None` while yielded to another pilot.
    pub active: Option<PlayerId>,
    /// Owner id of the nearest landed unpiloted remote ship within boarding reach.
    pub near_remote: Option<PlayerId>,
    /// Queued owner id for an outgoing `ShipAct::Board`.
    pub pending_board: Option<PlayerId>,
    /// Queued owner id for an outgoing `ShipAct::Leave`.
    pub pending_leave: Option<PlayerId>,
    /// Server approval waiting to seat the player in `board::board`.
    pub granted: Option<BoardGrant>,
    /// True when the local `Ship` must hide and disable its collider because someone else boarded it.
    pub yield_local: bool,
}
