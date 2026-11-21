//! The ship in the save file: its record is taken the moment the world is saved, and read back
//! when the world starts.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::{Beside, Ship, ShipState};
use super::frames::planet_velocity;
use super::sky::Skies;
use super::warp::drop_out;
use super::wells::Well;
use crate::saves::ActiveWorld;
use crate::space::{Clock, Frame, Location, Travel};
use crate::universe::{PlanetSpec, SystemSpec};

/// Returns the ship record stored in a world, when it has one that still reads.
pub fn saved(active: Option<&ActiveWorld>) -> Option<ShipState> {
    active?.0.save.ship.clone()?.try_into().ok()
}

/// Returns the record of `ship` among the bodies `wells`, with the player `seated` in it or not.
///
/// A ship in warp is recorded dropped out, at rest relative to the body it is beside, and a spooling
/// drive is recorded off: a loaded world never starts mid-jump.
pub fn record(ship: &Ship, wells: &[Well], seated: bool) -> ShipState {
    let mut still = ship.clone();
    drop_out(&mut still, wells);
    ShipState {
        seated,
        ..still.state()
    }
}

/// Returns where the ship of record `state` is against the body of `system` whose Hill sphere
/// holds it at `time` seconds, the smallest one when a moon and its planet both do. Nothing on
/// a surface, between the stars, in another system or outside every Hill sphere.
pub fn beside(state: &ShipState, system: &SystemSpec, time: f64) -> Option<Beside> {
    let (pos, vel) = (DVec3::from(state.pos), DVec3::from(state.vel));
    if state.frame != Frame::System(system.star.id) {
        return None;
    }
    let reach = |planet: &&PlanetSpec| planet.position(time).distance(pos) / planet.hill_radius();
    let held = system.planets.iter().filter(|planet| reach(planet) < 1.0);
    let body = held.min_by(|a, b| reach(a).total_cmp(&reach(b)))?;
    Some(Beside {
        body: body.id,
        pos: (pos - body.position(time)).to_array(),
        vel: (vel - planet_velocity(body, time)).to_array(),
        clock: time,
    })
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
    /// Universe time, which says where the bodies are.
    clock: Option<Res<'w, Clock>>,
}

impl Logbook<'_, '_> {
    /// Returns the record of the ship for a save file, or nothing without a ship. A ship beside
    /// a planet or a moon is recorded against it too, so it comes back beside it wherever the
    /// body has gone since.
    pub fn record(&self) -> Option<toml::Value> {
        let ship = self.ship.single().ok()?;
        let wells = self.skies.as_deref().map_or(&[][..], |skies| &skies.wells);
        let seated = self
            .location
            .as_deref()
            .is_some_and(|location| location.travel == Travel::InShip);
        let state = record(ship, wells, seated);
        let system = self.skies.as_deref().and_then(Skies::spec);
        let now = system.zip(self.clock.as_deref());
        let beside = now.and_then(|(system, clock)| beside(&state, system, clock.seconds));
        toml::Value::try_from(ShipState { beside, ..state }).ok()
    }
}
