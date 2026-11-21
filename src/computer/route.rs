//! The route: where the computer is set to go and how far along the ship is.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::atlas::Atlas;
use crate::settings::{KeyLabels, Keymap};
use crate::universe::{BodyId, GalaxyId, StarId, Universe};

/// Something the ship can be sent to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    /// A planet.
    Body(BodyId),
    /// A star, which stands for its whole system.
    Star(StarId),
    /// A galaxy.
    Galaxy(GalaxyId),
}

/// What the home planet is called after its name, wherever the player can pick it.
pub const HOME_LABEL: &str = "(Home)";

impl Target {
    /// Returns the display name of the target. The home planet, where the base stands, says so.
    pub fn name(self, universe: &Universe) -> String {
        let named = match self {
            Target::Body(id) if id == universe.home() => {
                Some(format!("{} {HOME_LABEL}", universe.planet(id).name))
            }
            Target::Body(id) => Some(universe.planet(id).name),
            Target::Star(id) => universe.star(id).map(|star| star.name),
            Target::Galaxy(id) => universe.galaxy(id).map(|galaxy| galaxy.name),
        };
        named.unwrap_or_else(|| "Unknown".to_string())
    }
}

/// Where the computer is set to go. Write this resource; the atlas copies it for the save.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Destination(pub Option<Target>);

/// Triggered when the player hands the destination to the autopilot.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Engage(pub Target);

/// Triggered when the player stops the route the autopilot flies: the drive drops out and the
/// ship is the pilot's again.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cancel;

/// Why the warp drive cannot take the course now, the first of the checks of the route that fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    /// The player is not in the ship. It comes before every other reason.
    OnFoot,
    /// The ship stands on the ground.
    Landed,
    /// The ship is inside the lock sphere of a planet or a star.
    Inside {
        /// True for a star, false for a planet or a moon.
        star: bool,
        /// Radii of the body the lock sphere reaches, in tenths.
        lock: u8,
        /// Radii from the centre the ship is at now, in tenths.
        tenths: u32,
    },
}

/// The names of the keys the sentences of the computer tell the player to press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    /// Boards the ship.
    pub board: String,
    /// Lifts the ship off the ground.
    pub rise: String,
    /// Sends the ship to the selected place, or stops the route it flies.
    pub go: String,
}

impl Keys {
    /// Returns the names of the keys of the bindings `map`, as `labels` prints them.
    pub fn of(map: &Keymap, labels: &KeyLabels) -> Keys {
        Keys {
            board: labels.label(map.ship.board),
            rise: labels.label(map.ship.rise),
            go: labels.label(map.engage),
        }
    }
}

impl Block {
    /// Returns the reason in plain words, with what to do about it and the keys to press.
    pub fn words(self, keys: &Keys) -> String {
        match self {
            Block::OnFoot => format!(
                "Board your ship first: walk to the hatch and press {}.",
                keys.board
            ),
            Block::Landed => format!("Take off first: hold {}.", keys.rise),
            Block::Inside { star, .. } => {
                let body = ["planet", "star"][usize::from(star)];
                format!(
                    "Too close to the {body}: fly away from it, then press {}.",
                    keys.go
                )
            }
        }
    }
}

/// How far along its route the ship is. The ship writes it, the computer shows it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RouteStatus {
    /// No route is being flown. With a course set, the autopilot can take it.
    #[default]
    Idle,
    /// A course is set or flown, but the drive cannot take it yet.
    Blocked(Block),
    /// The ship climbs out of the air of the planet it is over.
    Climbing,
    /// The drive, still running on the course before, drops out first.
    Dropping,
    /// The ship is turning toward the destination, this many percent of the way.
    Aligning(u8),
    /// The warp drive is charging.
    Spooling,
    /// The ship is in warp.
    InWarp,
    /// The ship has reached the destination.
    Arrived,
}

/// Copies a changed destination into the atlas so that it is saved with it.
pub(super) fn remember(destination: Res<Destination>, mut atlas: ResMut<Atlas>) {
    if atlas.destination != destination.0 {
        atlas.destination = destination.0;
    }
}
