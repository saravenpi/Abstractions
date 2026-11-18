//! The fog of the game camera: the haze of the air, the whiteout of the clouds and the water.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

use super::GameCamera;
use super::celestial::smoothstep;
use super::clouds::CloudCrossing;
use super::murk::{clearance, murk};
use super::palette::{Palette, srgb};
use super::vantage::{LocalSystem, Vantage};
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Frame, altitude};
use crate::world::{CHUNK_SIZE, Planet, Reach};

/// Fog and sky color seen from below the surface of water, in sRGB.
pub const UNDERWATER: Vec3 = Vec3::new(0.04, 0.27, 0.35);

/// Distance in meters at which the water hides everything.
const UNDERWATER_VISIBILITY: f32 = 25.0;

/// Heights above the ground in metres between which the fog opens from the edge of the detailed
/// ground to the horizon: shut on foot, open a few hundred metres up.
pub const OPEN: (f32, f32) = (15.0, 300.0);

/// Fog reach in horizon distances once open, so the horizon itself shows through light haze.
pub const HORIZON_SHARE: f32 = 2.5;

/// Share of the fog reach at which the linear fog hides everything.
pub const FOG_END: f32 = 0.9;

/// Distance in metres the camera culls at least, as Bevy does by default.
const NEAR_FAR: f32 = 1000.0;

/// Least share of air the fog still counts: above the air the fog reaches a thousand times farther.
pub const THIN_AIR: f32 = 1.0e-3;

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
/// the horizon, where the coarse planet still shows; it has no limit as the air runs out.
pub fn fog_sight(ground: f32, height: &Height) -> f32 {
    let up = height.altitude.max(0.0);
    let horizon = (up * (2.0 * height.radius + up)).sqrt();
    let open = smoothstep(OPEN.0, OPEN.1, height.clearance);
    let far = ground.max(horizon * HORIZON_SHARE);
    (ground + (far - ground) * open) / height.air.max(THIN_AIR)
}

/// Returns the distance in metres a camera that sees `reach` metres away culls at.
pub fn cull_distance(reach: f32) -> f32 {
    reach.max(NEAR_FAR)
}

/// Builds the camera fog for a key: dense fog in the colour of the liquid below its surface, or
/// linear fog that fades the terrain out before its reach.
pub(super) fn fog_for(key: FogKey) -> DistanceFog {
    match key.submerged {
        true => DistanceFog {
            color: srgb(key.color),
            falloff: FogFalloff::from_visibility(UNDERWATER_VISIBILITY),
            ..default()
        },
        false => DistanceFog {
            color: srgb(key.color),
            falloff: FogFalloff::Linear {
                start: key.reach * 0.35,
                end: key.reach * FOG_END,
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
    /// The star system around it, for the air of the planet below.
    local: Res<'w, LocalSystem>,
    /// How deep the camera is in the cloud layer.
    crossing: Res<'w, CloudCrossing>,
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// The planet the world streams, whose sea the camera may be under.
    planet: Option<Res<'w, Planet>>,
}

impl Depths<'_> {
    /// Returns the colour of the liquid around the render position `camera` when it is under
    /// the sea of the streamed planet, or nothing.
    ///
    /// The height comes from [`altitude`], which is this frame's, on the planet the anchor is
    /// fixed to. Away from its surface, and on a planet without a sea, nothing is submerged.
    fn submerged(&self, camera: Vec3) -> Option<Vec3> {
        let (anchor, planet) = (self.anchor.as_deref()?, self.planet.as_deref()?);
        let here = anchor.flat || anchor.frame == Frame::Surface(planet.body);
        let under = here && altitude(anchor, &planet.globe, camera) < 0.0;
        murk(planet.record()).filter(|_| under)
    }

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

/// Matches the fog to the sky horizon and the height of the camera, to the whiteout inside the
/// cloud layer, or to dense water fog below the surface, and culls the view where the fog ends.
fn update_fog(
    palette: Res<Palette>,
    settings: Res<Persistent<Settings>>,
    depths: Depths,
    mut cameras: Query<(&GlobalTransform, &mut DistanceFog, &mut Projection), With<GameCamera>>,
    mut last: Local<Option<FogKey>>,
) {
    let (vantage, crossing) = (&depths.vantage, &depths.crossing);
    let reach = Reach::of(Some(settings.graphics.render_distance));
    let planet = depths.planet.as_deref();
    let ground = planet.map_or(reach.rings as f32 * CHUNK_SIZE, |p| reach.sight(&p.globe));
    for (at, mut fog, mut projection) in &mut cameras {
        let camera = at.translation();
        let height = Height {
            altitude: vantage.altitude,
            clearance: depths.clearance(camera, vantage.altitude),
            radius: vantage.radius,
            air: vantage.air * depths.local.air(vantage.body).haze(),
        };
        let clear = fog_sight(ground, &height);
        let liquid = depths.submerged(camera);
        let air = palette.horizon.lerp(palette.cloud, crossing.depth);
        let key = FogKey {
            submerged: liquid.is_some(),
            color: liquid.unwrap_or(air),
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
    app.add_systems(Update, update_fog);
}
