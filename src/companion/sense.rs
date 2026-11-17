//! What the companion senses: one resource other modules may write, filled here from the day
//! clock, the player's eyes and the ship.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::aboard::{Aboard, read};
use super::spot::{Probe, Stance};
use crate::render::{DayClock, GameCamera, daylight};
use crate::space::{Anchor, Frame, Location, Travel, altitude, up};
use crate::world::Terrain;
use crate::world::structures::Surroundings;

/// Longest frame time the companion simulates, in seconds, so a hitch never throws it around.
const MAX_STEP: f32 = 0.05;

/// Distance in meters within which the nearest structure is looked for.
const STRUCTURE_REACH: f32 = 60.0;

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
    /// Render position of the nearest structure or village, refreshed every frame.
    pub structure: Option<Vec3>,
    /// Count of discoveries worth a delighted face: add one for each. Needs a producer.
    pub delight: u32,
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
    /// The player's eyes.
    pub camera: Single<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    /// The structures the world has surveyed around the player.
    sites: Option<Res<'w, Surroundings>>,
}

impl Around<'_, '_> {
    /// Returns the frame time in seconds, capped so a hitch never throws the companion around.
    pub fn dt(&self) -> f32 {
        self.time.delta_secs().min(MAX_STEP)
    }

    /// Returns wrapped seconds of game time.
    pub fn now(&self) -> f32 {
        self.time.elapsed_secs_wrapped()
    }

    /// Returns true while the player is seated in the ship.
    pub fn seated(&self) -> bool {
        self.location.travel == Travel::InShip
    }

    /// Returns where the player's eyes are and look. A seated player's vertical is the cockpit's.
    pub fn stance(&self) -> Stance {
        let eye = self.camera.translation();
        let vertical = match self.seated() {
            true => self.camera.up().as_vec3(),
            false => up(&self.anchor, eye),
        };
        Stance {
            eye,
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

    /// Returns the altitude of the eyes above sea level, or nothing away from a planet surface.
    pub fn altitude(&self) -> Option<f32> {
        matches!(self.anchor.frame, Frame::Surface(_)).then(|| {
            altitude(
                &self.anchor,
                self.terrain.globe(),
                self.camera.translation(),
            )
        })
    }
}

/// Returns the render position of the structure or village nearest to the eyes, for
/// [`CompanionSense::structure`], from what the world has surveyed around the player.
fn nearest_structure(around: &Around) -> Option<Vec3> {
    let Frame::Surface(_) = around.anchor.frame else {
        return None;
    };
    let eye = around.anchor.to_frame(around.camera.translation());
    let near = around.sites.as_deref()?.nearest(eye, STRUCTURE_REACH)?;
    Some(around.anchor.to_render(near.pos))
}

/// Fills the sensed values: night from the day clock, depth from the eyes, and the seat, the
/// climb and the clouds from the ship.
pub fn fill(
    around: Around,
    aboard: Aboard,
    clock: Res<DayClock>,
    mut sense: ResMut<CompanionSense>,
    mut was: Local<Option<f64>>,
) {
    let flight = aboard.flight(around.seated());
    let (depth, climb, in_clouds) = read(around.altitude(), *was, &flight, around.dt());
    *was = flight.height;
    let next = CompanionSense {
        night: 1.0 - daylight(clock.phase),
        seated: flight.seated,
        depth,
        climb,
        in_clouds,
        structure: nearest_structure(&around),
        delight: sense.delight,
    };
    sense.set_if_neq(next);
}
