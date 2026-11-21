//! The fog of the game camera: the aerial perspective of the air, and the whiteout of the clouds.
//!
//! The fog is the air and nothing else. It never hides the edge of the streamed ground: the
//! coarse terrain of the planet stands behind the detailed ground out to the horizon, and the air
//! fades it into the colour of the horizon by the rule of [`super::aerial`].

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

use super::aerial::{CONTRAST, Haze, sight};
use super::air::Air;
use super::clouds::CloudCrossing;
use super::exposure::{Metered, lifted};
use super::murk::clearance;
use super::palette::{Palette, srgb};
use super::vantage::{LocalSystem, Vantage};
use super::weather::{LocalWeather, WeatherLook};
use super::{GameCamera, SunDir};
use crate::space::{Anchor, Frame};
use crate::world::Planet;
use crate::world::globe::far::SIGHT;

/// Fog and sky color seen from below the surface of water, in sRGB. The underwater pass draws
/// it, so the distance fog of the camera stays the fog of the air.
pub const UNDERWATER: Vec3 = Vec3::new(0.04, 0.27, 0.35);

/// Distance in meters at which the water hides everything.
pub(super) const UNDERWATER_VISIBILITY: f32 = 25.0;

/// Distance in metres the camera culls at least, as Bevy does by default.
const NEAR_FAR: f32 = 1000.0;

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
    extinction: Vec3,
}

impl FogKey {
    /// Builds the key of the sRGB `color` for a camera in clear home air at sea level, before a
    /// planet is known.
    pub(super) fn home(color: Vec3) -> Self {
        Self {
            color,
            extinction: Haze::HOME.toward(0.0, 0.0),
        }
    }
}

/// Returns the share of red, green and blue light taken per metre from what a camera sees,
/// `altitude` metres above sea level and `clearance` metres above the ground under it, `cloud`
/// deep in the cloud layer from 0 to 1: the mean of `haze` between the camera and the ground
/// under it, so a camera on a plateau sees through the thin air of the plateau and a camera high
/// up sees the ground through the whole depth of the air, and the whiteout of the cloud.
pub fn extinction(haze: &Haze, (altitude, clearance): (f32, f32), cloud: f32) -> Vec3 {
    let ground = altitude - clearance.max(0.0);
    let white = CONTRAST / WHITEOUT_VISIBILITY * cloud.clamp(0.0, 1.0);
    haze.toward(altitude, ground) + Vec3::splat(white)
}

/// Returns the distance in metres a camera culls at over a planet of `radius` metres when the
/// air takes `extinction` per metre: where the air has hidden everything, at most
/// [`SIGHT`] planet radii, where nothing of the world is drawn any more, and no closer than
/// Bevy's default. The depth of the camera has no far plane, so this only culls.
pub fn cull_distance(extinction: Vec3, radius: f32) -> f32 {
    sight(extinction).min(SIGHT as f32 * radius).max(NEAR_FAR)
}

/// Builds the camera fog for a key: each colour of a far thing is taken at its own rate and
/// replaced by as much of the colour of the horizon.
pub(super) fn fog_for(key: FogKey) -> DistanceFog {
    DistanceFog {
        color: srgb(key.color),
        falloff: FogFalloff::Atmospheric {
            extinction: key.extinction,
            inscattering: key.extinction,
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
    /// Where the star stands, for the sky at the ground under the camera.
    sun: Res<'w, SunDir>,
    /// The exposure of the camera, which the unlit colour of the fog is lifted by.
    metered: Res<'w, Metered>,
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

    /// Returns the sRGB colour the air gives back for what it takes from a far thing: the horizon
    /// of the sky at the ground under the camera, through `air` under the weather `look`. It is
    /// the horizon the dome draws for a camera on the ground, and it stays the colour of the low
    /// air for a camera that has climbed out of it, so the ground seen from high up keeps its veil.
    fn colour(&self, air: &Air, look: &WeatherLook) -> Vec3 {
        let (ground, _) = Palette::under(&self.sun, &self.vantage, &self.local);
        lifted(look.sky(ground).horizon * air.share(), self.metered.gain())
    }
}

/// Gives the camera the fog of the air around it and under it, in the colour of the horizon,
/// closer under rain and dust, and the whiteout inside the cloud layer, and culls the view where
/// the air has hidden everything.
fn update_fog(
    palette: Res<Palette>,
    mut depths: Depths,
    mut cameras: Query<(&GlobalTransform, &mut DistanceFog, &mut Projection), With<GameCamera>>,
    mut last: Local<Option<FogKey>>,
) {
    let vantage = *depths.vantage;
    let (air, look) = (depths.local.air_below(&vantage), depths.weather.look());
    let haze = Haze::of(&air, vantage.atmosphere, &look);
    let cloud = depths.crossing.depth;
    let color = depths.colour(&air, &look).lerp(palette.cloud, cloud);
    for (at, mut fog, mut projection) in &mut cameras {
        let clearance = depths.clearance(at.translation(), vantage.altitude);
        depths.aloft.set_if_neq(Clearance(clearance));
        let key = FogKey {
            color,
            extinction: extinction(&haze, (vantage.altitude, clearance), cloud),
        };
        if *last != Some(key) {
            *last = Some(key);
            *fog = fog_for(key);
            if let Projection::Perspective(perspective) = &mut *projection {
                perspective.far = cull_distance(key.extinction, vantage.radius);
            }
        }
    }
}

/// Registers the fog update.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Clearance>()
        .add_systems(Update, update_fog);
}
