//! The ship in the save file: its record is taken the moment the world is saved, and read back
//! when the world starts.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::body::{Ship, ShipState};
use super::sky::Skies;
use super::warp::halt;
use super::wells::Well;
use crate::saves::ActiveWorld;
use crate::space::{Location, Travel};

/// Returns the ship record stored in a world, when it has one that still reads.
pub fn saved(active: Option<&ActiveWorld>) -> Option<ShipState> {
    active?.0.save.ship.clone()?.try_into().ok()
}

/// Returns the record of `ship` among the bodies `wells`, with the player `seated` in it or not.
///
/// A ship in warp is recorded dropped out, at rest relative to the nearest body, and a spooling
/// drive is recorded off: a loaded world never starts mid-jump.
pub fn record(ship: &Ship, wells: &[Well], seated: bool) -> ShipState {
    let mut still = ship.clone();
    halt(&mut still, wells);
    ShipState {
        seated,
        ..still.state()
    }
}

/// The ship as a save sees it. The save system holds this parameter and asks for the record
/// when it writes, so the record and the universe time of a save belong to the same instant.
#[derive(SystemParam)]
pub struct Logbook<'w, 's> {
    /// The ship, when the world has one.
    ship: Query<'w, 's, &'static Ship>,
    /// What surrounds the ship.
    skies: Option<Res<'w, Skies>>,
    /// Whether the player is seated.
    location: Option<Res<'w, Location>>,
}

impl Logbook<'_, '_> {
    /// Returns the record of the ship for a save file, or nothing without a ship.
    pub fn record(&self) -> Option<toml::Value> {
        let ship = self.ship.single().ok()?;
        let wells = self.skies.as_deref().map_or(&[][..], |skies| &skies.wells);
        let seated = self
            .location
            .as_deref()
            .is_some_and(|location| location.travel == Travel::InShip);
        toml::Value::try_from(record(ship, wells, seated)).ok()
    }
}
