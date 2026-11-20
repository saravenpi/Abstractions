//! Feeding the cockpit of the ship the player flies: the controls from its helm and its state,
//! the instruments from the same telemetry the HUD reads, and the pilot from the seat.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::attitude::attitude;
use super::controls::Levers;
use super::readings::{LAMPS, Lamp, Readings};
use super::state::Cockpit;
use super::tune::{FRAME_LAMPS, HOT_LAMP};
use crate::computer::{Destination, ship_position, target_position};
use crate::condition::Condition;
use crate::player::Player;
use crate::render::Heat;
use crate::ship::body::Ship;
use crate::ship::helm::Helm;
use crate::ship::sim::{Cosmos, Pace};
use crate::ship::sky::Skies;
use crate::ship::telemetry::ShipTelemetry;
use crate::ship::warp::{Warp, clearance};
use crate::ship::wells::beside;
use crate::sim::PlayerId;
use crate::space::{Anchor, Frame, Location, Placed, Travel, up};

/// Returns where the warp switch stands for the state of the drive: off, spooling, engaged.
pub fn switch(warp: Warp) -> f32 {
    match warp {
        Warp::Off => 0.0,
        Warp::Spooling { .. } => 0.5,
        Warp::Active { .. } => 1.0,
    }
}

/// What the instruments are read from, besides the ship.
#[derive(SystemParam)]
pub struct Flight<'w> {
    /// The instruments the HUD reads.
    telemetry: Res<'w, ShipTelemetry>,
    /// What the pilot asks.
    helm: Res<'w, Helm>,
    /// The bodies around the ship.
    skies: Res<'w, Skies>,
    /// What the last flight step did.
    pace: Res<'w, Pace>,
    /// The render anchor.
    anchor: Res<'w, Anchor>,
    /// The course that is set.
    destination: Res<'w, Destination>,
    /// The universe and its clock.
    cosmos: Cosmos<'w>,
    /// The heat of an entry.
    heat: Option<Res<'w, Heat>>,
    /// The condition of the frame.
    condition: Option<Res<'w, Condition>>,
    /// Where the player is.
    location: Res<'w, Location>,
}

impl Flight<'_> {
    /// Returns the pitch and the roll of the ship against the local up, while the anchor is the
    /// surface of a planet: in space there is no horizon.
    fn attitude(&self, placed: &Placed) -> Option<Vec2> {
        let Frame::Surface(_) = self.anchor.frame else {
            return None;
        };
        let turn = self.anchor.rotation_to_render(placed.rot);
        let up = up(&self.anchor, self.anchor.to_render(placed.pos));
        attitude(turn.inverse() * up)
    }

    /// Returns the unit direction from the ship to the destination of the course, in body axes.
    fn course(&self, ship: &Ship) -> Option<Vec3> {
        let target = self.destination.0?;
        let (universe, time) = (&self.cosmos.universe, self.cosmos.clock.seconds);
        let here = ship_position(universe, ship.frame, ship.pos, time)?;
        let to = target_position(universe, target, time)? - here;
        (ship.rot.inverse() * to)
            .try_normalize()
            .map(|d| d.as_vec3())
    }

    /// Returns the unit direction from the ship to the body it is beside, in body axes, away
    /// from a surface, where the bodies are in the frame of the ship.
    fn body(&self, ship: &Ship) -> Option<Vec3> {
        let Frame::System(_) = ship.frame else {
            return None;
        };
        let well = beside(&self.skies.wells, ship.pos)?;
        let to = ship.rot.inverse() * (well.pos - ship.pos);
        to.try_normalize().map(|d| d.as_vec3())
    }

    /// Returns which lamps are lit, from the `lamps` the telemetry lights: it adds the lock of a
    /// body, the heat of an entry and the integrity of the frame from their own sources.
    fn lamps(&self, ship: &Ship, mut lamps: [bool; LAMPS]) -> [bool; LAMPS] {
        let locked = matches!(ship.frame, Frame::System(_))
            && !ship.landed
            && clearance(&self.skies.wells, ship.pos) <= 0.0;
        let hot = self
            .heat
            .as_ref()
            .is_some_and(|heat| heat.glow.heat > HOT_LAMP);
        let integrity = self.condition.as_ref().map_or(1.0, |c| c.integrity());
        lamps[Lamp::Close as usize] = locked;
        lamps[Lamp::Hot as usize] = hot;
        lamps[Lamp::Frame as usize] = integrity < FRAME_LAMPS.0;
        lamps[Lamp::Failing as usize] = integrity < FRAME_LAMPS.1;
        lamps
    }

    /// Returns everything the instruments of `ship` read.
    fn readings(&self, ship: &Ship, placed: &Placed) -> Readings {
        let base = Readings::of(&self.telemetry, ship.gear);
        Readings {
            lamps: self.lamps(ship, base.lamps),
            attitude: self.attitude(placed),
            course: self.course(ship),
            body: self.body(ship),
            accel: (ship.rot.inverse() * self.pace.report.proper).as_vec3(),
            ..base
        }
    }
}

/// Feeds the cockpit of the local ship: the stick is the real pitch, yaw and roll input,
/// whoever gives it, the lever the real throttle, the gear lever and the warp switch the real
/// gear and drive, the instruments what they measure, and the pilot the local player while
/// seated.
pub fn feed(
    flight: Flight,
    players: Query<(Entity, &PlayerId), With<Player>>,
    ship: Single<(&Ship, &Placed, &mut Cockpit)>,
) {
    let (ship, placed, mut cockpit) = ship.into_inner();
    let local = players.iter().find(|(_, id)| **id == PlayerId::LOCAL);
    let seated = flight.location.travel == Travel::InShip;
    let fed = Cockpit {
        asked: Levers {
            stick: flight.helm.controls.turn.as_vec3(),
            throttle: ship.throttle as f32,
            gear: ship.gear as f32,
            warp: switch(ship.warp),
        },
        read: flight.readings(ship, placed),
        pilot: local.filter(|_| seated).map(|(player, _)| player),
        ..*cockpit
    };
    cockpit.set_if_neq(fed);
}
