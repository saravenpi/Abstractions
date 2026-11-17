//! The route: where the computer is set to go and how far along the ship is.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::atlas::Atlas;
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

/// How far along its route the ship is. The ship writes it, the computer shows it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RouteStatus {
    /// No route is being flown.
    #[default]
    Idle,
    /// The ship is turning toward the destination.
    Aligning,
    /// The warp drive is charging.
    Spooling,
    /// The ship is in warp.
    InWarp,
    /// The ship has reached the destination.
    Arrived,
}

impl RouteStatus {
    /// Returns the words shown for this state.
    pub fn label(self) -> &'static str {
        match self {
            RouteStatus::Idle => "idle",
            RouteStatus::Aligning => "aligning",
            RouteStatus::Spooling => "spooling",
            RouteStatus::InWarp => "in warp",
            RouteStatus::Arrived => "arrived",
        }
    }
}

/// Copies a changed destination into the atlas so that it is saved with it.
pub(super) fn remember(destination: Res<Destination>, mut atlas: ResMut<Atlas>) {
    if atlas.destination != destination.0 {
        atlas.destination = destination.0;
    }
}
