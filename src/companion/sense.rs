//! What the companion senses: one resource other modules may write, filled here as far as public state allows.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::spot::{Probe, Stance};
use crate::render::{DayClock, GameCamera, daylight};
use crate::space::{Anchor, Frame, Location, Travel, altitude, up};
use crate::universe::{BodyId, Universe};
use crate::world::Terrain;

/// Longest frame time the companion simulates, in seconds, so a hitch never throws it around.
const MAX_STEP: f32 = 0.05;

/// What happens around the player, as far as the companion cares.
///
/// `fill` writes every field but `structure` and `delight` each frame. Those two wait for a
/// producer and keep their defaults until one writes them.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct CompanionSense {
    /// How dark it is, from 0 (day) to 1 (night). Filled from the day clock.
    pub night: f32,
    /// Depth of the player's eyes under the sea surface in meters, zero above it. Filled from the camera.
    pub depth: f32,
    /// True while the player is seated in the ship. Filled from `Location`.
    pub seated: bool,
    /// Climb rate of the seated player in meters per second. Filled from the camera altitude.
    pub climb: f32,
    /// True while the seated player crosses the cloud layer. Filled from the planet's cloud heights.
    pub in_clouds: bool,
    /// Render position of the nearest structure or village, refreshed every frame. Needs a producer.
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

/// Returns the depth under water, the climb rate and whether the clouds are being crossed.
///
/// `was` and `now` are the altitudes of the eyes on the last frame and this one, `clouds` the base
/// and the top of the cloud layer. Climbing and clouds only count while `seated`.
pub fn read(
    was: Option<f32>,
    now: Option<f32>,
    clouds: (f32, f32),
    seated: bool,
    dt: f32,
) -> (f32, f32, bool) {
    let Some(height) = now else {
        return (0.0, 0.0, false);
    };
    let climb = was.map_or(0.0, |was| (height - was) / dt.max(1e-4));
    let cloudy = (clouds.0..=clouds.1).contains(&height);
    (
        (-height).max(0.0),
        if seated { climb } else { 0.0 },
        seated && cloudy,
    )
}

/// What `fill` remembers between frames.
#[derive(Default)]
pub struct Memo {
    /// Altitude of the eyes on the last frame.
    was: Option<f32>,
    /// The planet the cloud heights were read for, with its cloud base and top in meters.
    clouds: Option<(BodyId, (f32, f32))>,
}

/// Fills the sensed values that public resources give: night, depth, seat, climb and clouds.
pub fn fill(
    around: Around,
    clock: Res<DayClock>,
    universe: Res<Universe>,
    mut sense: ResMut<CompanionSense>,
    mut memo: Local<Memo>,
) {
    let body = around.location.body;
    let (_, clouds) = *memo
        .clouds
        .filter(|(known, _)| *known == body)
        .get_or_insert_with(|| {
            let spec = universe.planet(body);
            (body, (spec.cloud_base as f32, spec.cloud_top as f32))
        });
    memo.clouds = Some((body, clouds));
    let (now, seated) = (around.altitude(), around.seated());
    let (depth, climb, in_clouds) = read(memo.was, now, clouds, seated, around.dt());
    memo.was = now;
    sense.night = 1.0 - daylight(clock.phase);
    sense.seated = seated;
    sense.depth = depth;
    sense.climb = climb;
    sense.in_clouds = in_clouds;
}
