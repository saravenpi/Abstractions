//! What the companion senses: one resource other modules may write, filled here from the day
//! clock, the player's eyes and the ship.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Companion;
use super::aboard::{Aboard, read};
use super::spot::{Probe, Stance};
use super::tune::MAX_STEP;
use super::wing::aloft;
use crate::gear::Gear;
use crate::player::{EYE_OFFSET, Player, above_sea};
use crate::render::{DayClock, GameCamera, daylight};
use crate::space::{Anchor, Frame, Location, Travel, up};
use crate::world::Terrain;

/// What happens around the player, as far as the companion cares.
///
/// `fill` writes every field each frame but `delight`, which a producer raises.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct CompanionSense {
    /// How dark it is, from 0 (day) to 1 (night). Filled from the day clock.
    pub night: f32,
    /// Depth of the player's eyes under the sea surface in meters, zero above it. Filled from the camera.
    pub depth: f32,
    /// True while the player is seated in the ship. Filled from `Location`.
    pub seated: bool,
    /// Climb rate of the flying ship in meters per second while the player is seated. Filled
    /// from the ship's altimeter.
    pub climb: f32,
    /// True while the seated player crosses the cloud layer. Filled from the sky's cloud crossing.
    pub in_clouds: bool,
    /// Count of discoveries worth a delighted face: add one for each. Needs a producer.
    pub delight: u32,
    /// True while the flight gear carries the player through the air. Filled from the gear.
    pub flying: bool,
    /// True while the player dives steeply and fast. Filled from the gear.
    pub diving: bool,
}

/// The world around the player that every companion system reads.
#[derive(SystemParam)]
pub struct Around<'w, 's> {
    /// Frame time.
    time: Res<'w, Time>,
    /// The render anchor.
    pub anchor: Res<'w, Anchor>,
    /// The terrain of the anchored planet.
    pub terrain: Res<'w, Terrain>,
    /// Where the player is.
    pub location: Res<'w, Location>,
    /// The game camera, which tells where the player looks.
    pub camera: Single<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    /// The player body. On foot the eyes are above its centre; seated it sits at the eye point
    /// of the pilot, turned with the ship.
    pub body: Single<'w, 's, &'static Transform, (With<Player>, Without<Companion>)>,
}

impl Around<'_, '_> {
    /// Returns the frame time in seconds, capped so a hitch never throws the companion around.
    pub fn dt(&self) -> f32 {
        self.time.delta_secs().min(MAX_STEP)
    }

    /// Returns true while the player is seated in the ship.
    pub fn seated(&self) -> bool {
        self.location.travel == Travel::InShip
    }

    /// Returns where the player's eyes are and look, from the body, so that a camera behind it
    /// changes nothing. Seated, the eyes, the vertical and the view are the cockpit's.
    pub fn stance(&self) -> Stance {
        let body = *self.body;
        if self.seated() {
            return Stance {
                eye: body.translation,
                forward: body.forward().as_vec3(),
                up: body.up().as_vec3(),
            };
        }
        let vertical = up(&self.anchor, body.translation);
        Stance {
            eye: body.translation + vertical * EYE_OFFSET,
            forward: self.camera.forward().as_vec3(),
            up: vertical,
        }
    }

    /// Returns the reader of the ground and the water.
    pub fn probe(&self) -> Probe<'_> {
        Probe {
            anchor: &self.anchor,
            terrain: &self.terrain,
        }
    }

    /// Returns the height of the eyes above the liquid surface, infinite on a dry world, or nothing away
    /// from a planet surface.
    pub fn afloat(&self) -> Option<f32> {
        matches!(self.anchor.frame, Frame::Surface(_))
            .then(|| above_sea(&self.anchor, self.terrain.globe(), self.stance().eye))
    }
}

/// Fills the sensed values: night from the day clock, depth from the eyes, the seat, the
/// climb and the clouds from the ship, and the flight from the gear.
pub fn fill(
    around: Around,
    aboard: Aboard,
    (clock, gear): (Res<DayClock>, Query<&Gear, With<Player>>),
    mut sense: ResMut<CompanionSense>,
    mut was: Local<Option<f64>>,
) {
    let state = gear.single().map(|gear| gear.state).unwrap_or_default();
    let (flying, diving) = aloft(&state);
    let flight = aboard.flight(around.seated());
    let (depth, climb, in_clouds) = read(around.afloat(), *was, &flight, around.dt());
    *was = flight.height;
    let next = CompanionSense {
        night: 1.0 - daylight(clock.phase),
        seated: flight.seated,
        depth,
        climb,
        in_clouds,
        delight: sense.delight,
        flying: flying && !flight.seated,
        diving: diving && !flight.seated,
    };
    sense.set_if_neq(next);
}
