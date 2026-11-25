//! Chests, felled trees, terrain digs, and gunfire shots on the reliable wire.

use serde::{Deserialize, Serialize};

use crate::chest::{ChestItem, PlaceableKind, PlacedProp};
use crate::ids::BodyId;
use crate::log::PlayerId;

/// Most placed chests and torches sent in one join snapshot message.
pub const CHEST_CHUNK: usize = 16;

/// Most felled trees sent in one join snapshot message.
pub const TREE_CHUNK: usize = 64;

/// Most terrain dig craters sent in one join snapshot message.
pub const DIG_CHUNK: usize = 64;

/// Most bytes allowed in a felled tree identifier on the wire.
pub const TREE_ID_MAX: usize = 96;

/// Most terrain dig craters retained per world on the server.
pub const DIGS_MAX: usize = 8192;

/// Most felled trees retained per world on the server.
pub const TREES_MAX: usize = 8192;

/// What a player asks the server to do with a placed chest or torch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChestAct {
    /// Place a new chest or torch on `body` at unit direction `dir` and `yaw`.
    Place {
        /// Body the prop sits on.
        body: BodyId,
        /// Surface direction from the body centre in its rotating frame.
        dir: [f32; 3],
        /// Yaw orientation around the local up axis, in radians.
        yaw: f32,
        /// Kind of placeable prop.
        kind: PlaceableKind,
    },
    /// Remove placed prop `id` from the world.
    Break {
        /// World id of the prop to remove.
        id: u64,
    },
    /// Set slot `slot` of chest `id` to `item` (or empty with `None`).
    SetSlot {
        /// World id of the chest.
        id: u64,
        /// Zero-based slot index inside the chest.
        slot: u8,
        /// New slot item, or `None` to clear.
        item: Option<ChestItem>,
    },
    /// Sort and merge slots of chest `id`.
    Sort {
        /// World id of the chest.
        id: u64,
    },
}

/// What the server tells players about placed chests and torches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChestEvent {
    /// A new chest or torch was placed with server-assigned `id`.
    Placed(PlacedProp),
    /// Placed prop `id` was broken by player `by`.
    Broken {
        /// World id of the broken prop.
        id: u64,
        /// Player who broke it.
        by: PlayerId,
    },
    /// Slot `slot` of chest `id` changed to `item`.
    Slot {
        /// World id of the chest.
        id: u64,
        /// Zero-based slot index inside the chest.
        slot: u8,
        /// Updated slot item, or `None` when cleared.
        item: Option<ChestItem>,
    },
    /// Chest `id` was sorted and now holds `slots`.
    Sorted {
        /// World id of the chest.
        id: u64,
        /// Full updated slot list of the chest.
        slots: Vec<Option<ChestItem>>,
    },
}

/// A tree felled by a player on a body.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TreeFell {
    /// Body where the tree stood.
    pub body: BodyId,
    /// Stable tree identifier string.
    pub id: String,
}

/// A terrain crater stamp carved by a player on a body.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DigOp {
    /// Body where the crater was dug.
    pub body: BodyId,
    /// Surface direction from the body centre in its rotating frame.
    pub dir: [f32; 3],
    /// Crater radius along the surface, in metres.
    pub radius: f32,
    /// Peak crater depth at the centre, in metres.
    pub depth: f32,
}

/// A firearm shot fired by a player for remote visual and audio replay.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Shot {
    /// Body where the shot was fired.
    pub body: BodyId,
    /// Muzzle origin in the body frame.
    pub origin: [f32; 3],
    /// Shot travel direction in the body frame.
    pub dir: [f32; 3],
    /// Firearm kind tag.
    pub kind: u8,
    /// Barrel material tag.
    pub barrel: u8,
    /// Ammunition kind tag.
    pub ammo: u8,
}
