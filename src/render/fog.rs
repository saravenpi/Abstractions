//! The fog of the game camera: the haze of the air, the whiteout of the clouds and the water.

use bevy::ecs::system::SystemParam;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

use super::GameCamera;
use super::clouds::CloudCrossing;
use super::palette::{Palette, srgb};
use super::vantage::Vantage;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Frame, altitude};
use crate::world::{CHUNK_SIZE, Planet, Reach};

/// Fog and sky color seen from below the surface, in sRGB.
pub const UNDERWATER: Vec3 = Vec3::new(0.04, 0.27, 0.35);

/// Distance in meters at which the water hides everything.
const UNDERWATER_VISIBILITY: f32 = 25.0;

/// Height above sea level in meters, over every summit, from which the fog starts to open up.
const CLIMB_FLOOR: f32 = 300.0;

/// Meters of sight gained per meter climbed above that height, so the planet below shows from high up.
const CLIMB_SIGHT: f32 = 10.0;

/// Least share of air the fog still counts: above the air the fog reaches a thousand times farther.
const THIN_AIR: f32 = 1.0e-3;

/// Distance in meters at which the middle of the cloud layer hides everything.
const WHITEOUT_VISIBILITY: f32 = 70.0;

/// What the fog looks like, so the camera is only written when it changes.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct FogKey {
    submerged: bool,
    color: Vec3,
    reach: f32,
}

impl FogKey {
    /// Builds the key for a camera in clear air on the ground.
    pub(super) fn ground(color: Vec3, render_distance: u32) -> Self {
        Self {
            submerged: false,
            color,
            reach: fog_reach(render_distance, 0.0, 1.0),
        }
    }
}

/// Returns the distance in meters the fog lets the camera see: the streamed terrain on foot,
/// farther with `altitude` once above every summit, and without limit as the share of `air` left
/// goes to zero.
pub fn fog_reach(render_distance: u32, altitude: f32, air: f32) -> f32 {
    fog_sight(render_distance as f32 * CHUNK_SIZE, altitude, air)
}

/// Returns the distance in meters the fog lets the camera see when the detail of the ground
/// ends `ground` meters away: farther with `altitude` once above every summit, and without limit
/// as the share of `air` left goes to zero.
pub fn fog_sight(ground: f32, altitude: f32, air: f32) -> f32 {
    (ground + (altitude - CLIMB_FLOOR).max(0.0) * CLIMB_SIGHT) / air.max(THIN_AIR)
}

/// Builds the camera fog for a key: dense water fog below the surface, or linear fog that fades
/// the terrain out before its reach.
pub(super) fn fog_for(key: FogKey) -> DistanceFog {
    match key.submerged {
        true => DistanceFog {
            color: srgb(UNDERWATER),
            falloff: FogFalloff::from_visibility(UNDERWATER_VISIBILITY),
            ..default()
        },
        false => DistanceFog {
            color: srgb(key.color),
            falloff: FogFalloff::Linear {
                start: key.reach * 0.35,
                end: key.reach * 0.9,
            },
            ..default()
        },
    }
}

/// Where the camera is, for the fog.
#[derive(SystemParam)]
struct Depths<'w> {
    /// Where the camera is in the sky.
    vantage: Res<'w, Vantage>,
    /// How deep the camera is in the cloud layer.
    crossing: Res<'w, CloudCrossing>,
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// The planet the world streams, whose sea the camera may be under.
    planet: Option<Res<'w, Planet>>,
}

impl Depths<'_> {
    /// Returns true when the render position `camera` is under the sea of the streamed planet.
    ///
    /// The height comes from [`altitude`], which is this frame's, on the planet the anchor is
    /// fixed to. Away from its surface nothing is under water.
    fn submerged(&self, camera: Vec3) -> bool {
        let (Some(anchor), Some(planet)) = (self.anchor.as_deref(), self.planet.as_deref()) else {
            return false;
        };
        let here = anchor.flat || anchor.frame == Frame::Surface(planet.body);
        here && altitude(anchor, &planet.globe, camera) < 0.0
    }
}

/// Matches the fog to the sky horizon and the height of the camera, to the whiteout inside the
/// cloud layer, or to dense water fog below the surface.
fn update_fog(
    palette: Res<Palette>,
    settings: Res<Persistent<Settings>>,
    depths: Depths,
    mut cameras: Query<(&GlobalTransform, &mut DistanceFog), With<GameCamera>>,
    mut last: Local<Option<FogKey>>,
) {
    let (vantage, crossing) = (&depths.vantage, &depths.crossing);
    let reach = Reach::of(Some(settings.graphics.render_distance));
    let planet = depths.planet.as_deref();
    let ground = planet.map_or(reach.rings as f32 * CHUNK_SIZE, |p| reach.sight(&p.globe));
    let clear = fog_sight(ground, vantage.altitude, vantage.air);
    for (at, mut fog) in &mut cameras {
        let key = FogKey {
            submerged: depths.submerged(at.translation()),
            color: palette.horizon.lerp(palette.cloud, crossing.depth),
            reach: clear * (WHITEOUT_VISIBILITY / clear).powf(crossing.depth),
        };
        if *last != Some(key) {
            *last = Some(key);
            *fog = fog_for(key);
        }
    }
}

/// Registers the fog update.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_fog);
}
