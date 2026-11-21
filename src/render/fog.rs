//! The fog of the game camera: the haze of the air, the whiteout of the clouds and the water.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

use super::GameCamera;
use super::celestial::smoothstep;
use super::clouds::CloudCrossing;
use super::murk::clearance;
use super::palette::{Palette, srgb};
use super::vantage::{LocalSystem, Vantage};
use super::weather::LocalWeather;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Frame};
use crate::world::globe::far::SIGHT;
use crate::world::{CHUNK_SIZE, Planet, Reach};

/// Fog and sky color seen from below the surface of water, in sRGB. The underwater pass draws
/// it, so the distance fog of the camera stays the fog of the air.
pub const UNDERWATER: Vec3 = Vec3::new(0.04, 0.27, 0.35);

/// Distance in meters at which the water hides everything.
pub(super) const UNDERWATER_VISIBILITY: f32 = 25.0;

/// Heights above the ground in metres between which the fog opens from the edge of the detailed
/// ground to the horizon: shut on foot, open a few hundred metres up.
pub const OPEN: (f32, f32) = (15.0, 300.0);

/// Fog reach in horizon distances once open, so the horizon itself shows through light haze.
pub const HORIZON_SHARE: f32 = 2.5;

/// Density of the fog over the reach: the share of the view the fog takes at a distance `d` is
/// `1 - exp(-(DENSITY d / reach)^2)`, 1 percent left at [`FOG_END`] of the reach and a third of a
/// percent at the reach, so the ground fades out smoothly with no wall where it is cut.
pub const FOG_DENSITY: f32 = 2.4;

/// Share of the fog reach at which the fog hides all but one percent of what is behind it.
pub const FOG_END: f32 = 0.9;

/// Distance in metres at which the clear home air hides a dark object to the 5 percent contrast
/// of the eye (Koschmieder, as Bevy's fog takes it): a clear day shows the far coast tens of
/// kilometres away. The clouds and the sea fade with it, and thicker air divides it.
pub const AIR_VISIBILITY: f32 = 40_000.0;

/// Distance in metres the camera culls at least, as Bevy does by default.
const NEAR_FAR: f32 = 1000.0;

/// Least share of air the fog still counts: above the air the fog reaches a thousand times farther.
pub const THIN_AIR: f32 = 1.0e-3;

/// Distance in meters at which the middle of the cloud layer hides everything.
const WHITEOUT_VISIBILITY: f32 = 70.0;

/// Height of the game camera above the ground or the sea under it, in metres, as the fog last
/// measured it. The shading reads it to take the ambient occlusion pass off a high camera.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Clearance(pub f32);

/// What the fog looks like, so the camera is only written when it changes.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct FogKey {
    color: Vec3,
    reach: f32,
}

impl FogKey {
    /// Builds the key for a camera in clear air on the ground.
    pub(super) fn ground(color: Vec3, render_distance: u32) -> Self {
        Self {
            color,
            reach: render_distance as f32 * CHUNK_SIZE,
        }
    }
}

/// Where the camera is, as the fog sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Height {
    /// Height above sea level in metres.
    pub altitude: f32,
    /// Height above the ground or the sea under the camera, in metres.
    pub clearance: f32,
    /// Sea level radius of the planet in metres.
    pub radius: f32,
    /// Share of a home atmosphere of haze left above the camera: 1 at the ground of the home
    /// planet, 0 in space and without air, above 1 in thicker air.
    pub air: f32,
}

/// Returns the distance in metres the fog lets the camera see when the detail of the ground ends
/// `ground` metres away. On foot that is the ground; climbing, the fog opens smoothly to beyond
/// the horizon, where the coarse planet still shows; as the air runs out it reaches on, up to
/// [`SIGHT`] planet radii, far past the hand-over.
pub fn fog_sight(ground: f32, height: &Height) -> f32 {
    let up = height.altitude.max(0.0);
    let horizon = (up * (2.0 * height.radius + up)).sqrt();
    let open = smoothstep(OPEN.0, OPEN.1, height.clearance);
    let far = ground.max(horizon * HORIZON_SHARE);
    let reach = (ground + (far - ground) * open) / height.air.max(THIN_AIR);
    reach.min(SIGHT as f32 * height.radius)
}

/// Returns the distance in metres a camera that sees `reach` metres away culls at.
pub fn cull_distance(reach: f32) -> f32 {
    reach.max(NEAR_FAR)
}

/// Builds the camera fog for a key: linear fog that fades the terrain out before its reach.
pub(super) fn fog_for(key: FogKey) -> DistanceFog {
    DistanceFog {
        color: srgb(key.color),
        falloff: FogFalloff::ExponentialSquared {
            density: FOG_DENSITY / key.reach.max(1.0),
        },
        ..default()
    }
}

/// Where the camera is, for the fog.
#[derive(SystemParam)]
struct Depths<'w> {
    /// Where the camera is in the sky.
    vantage: Res<'w, Vantage>,
    /// The star system around it, for the air of the planet below.
    local: Res<'w, LocalSystem>,
    /// How deep the camera is in the cloud layer.
    crossing: Res<'w, CloudCrossing>,
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// The planet the world streams, whose ground the camera stands over.
    planet: Option<Res<'w, Planet>>,
    /// The weather at the camera, whose rain and dust close the fog in.
    weather: Res<'w, LocalWeather>,
    /// The height above the ground the lights read.
    aloft: ResMut<'w, Clearance>,
}

impl Depths<'_> {
    /// Returns the height of the render position `camera` above the ground or the sea under it,
    /// which is `altitude` above sea level, on the streamed planet the anchor is fixed to.
    fn clearance(&self, camera: Vec3, altitude: f32) -> f32 {
        let (Some(anchor), Some(planet)) = (self.anchor.as_deref(), self.planet.as_deref()) else {
            return altitude;
        };
        if anchor.flat || anchor.frame != Frame::Surface(planet.body) {
            return altitude;
        }
        let dir = anchor.to_frame(camera).normalize_or(DVec3::Y);
        clearance(&planet.globe, dir, altitude)
    }
}

/// Matches the fog to the sky horizon and the height of the camera, closer under rain and dust,
/// and to the whiteout inside the cloud layer, and culls the view where the fog ends.
fn update_fog(
    palette: Res<Palette>,
    settings: Res<Persistent<Settings>>,
    mut depths: Depths,
    mut cameras: Query<(&GlobalTransform, &mut DistanceFog, &mut Projection), With<GameCamera>>,
    mut last: Local<Option<FogKey>>,
) {
    let (vantage, crossing) = (&depths.vantage, &depths.crossing);
    let reach = Reach::of(Some(settings.graphics.render_distance));
    let flat = reach.rings as f32 * CHUNK_SIZE;
    let ground = (depths.planet.as_deref()).map_or(flat, |p| reach.sight(&p.globe));
    let below = depths.local.air_below(vantage);
    for (at, mut fog, mut projection) in &mut cameras {
        let camera = at.translation();
        let height = Height {
            altitude: vantage.altitude,
            clearance: depths.clearance(camera, vantage.altitude),
            radius: vantage.radius,
            air: vantage.air * below.haze(),
        };
        depths.aloft.set_if_neq(Clearance(height.clearance));
        let clear = fog_sight(ground, &height) * depths.weather.look().sight();
        let key = FogKey {
            color: palette.horizon.lerp(palette.cloud, crossing.depth),
            reach: clear * (WHITEOUT_VISIBILITY / clear).powf(crossing.depth),
        };
        if *last != Some(key) {
            *last = Some(key);
            *fog = fog_for(key);
            if let Projection::Perspective(perspective) = &mut *projection {
                perspective.far = cull_distance(clear);
            }
        }
    }
}

/// Registers the fog update.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Clearance>()
        .add_systems(Update, update_fog);
}
