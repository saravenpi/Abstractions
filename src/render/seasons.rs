//! Repaints the colour lookup texture of the ground for the season at the place under the
//! camera, whenever the rounded look of the season changes. Every mesh that shows shares the
//! texture, so the season fades out as the camera leaves the ground.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::season::{SeasonLook, repaint};
use super::vantage::{LocalSystem, Vantage};
use crate::climate::{Setting, Spot};
use crate::space::Clock;
use crate::world::{Planet, Sited};

/// The colour lookup texture of the ground, its colours as the world made them, and the
/// season look it shows.
#[derive(Resource, Debug, Default)]
struct Lut {
    /// Material of the ground that samples the texture.
    material: Option<Handle<StandardMaterial>>,
    /// The texture.
    image: Option<Handle<Image>>,
    /// Its bytes before any season touched them.
    original: Vec<u8>,
    /// The season look it shows now.
    shown: Option<SeasonLook>,
}

/// Finds the lookup texture through the material of the first new ground entity whose material
/// is not the one already known, and keeps a copy of its colours.
fn find(
    new: Query<&MeshMaterial3d<StandardMaterial>, Added<Sited>>,
    materials: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
    mut lut: ResMut<Lut>,
) {
    for handle in &new {
        if lut.material.as_ref() == Some(&handle.0) {
            continue;
        }
        let texture = materials
            .get(&handle.0)
            .and_then(|m| m.base_color_texture.clone());
        let Some((texture, image)) = texture.and_then(|t| Some((t.clone(), images.get(&t)?)))
        else {
            continue;
        };
        let Some(data) = image.data.as_ref() else {
            continue;
        };
        *lut = Lut {
            material: Some(handle.0.clone()),
            image: Some(texture),
            original: data.clone(),
            shown: None,
        };
        return;
    }
}

/// Seconds between two readings of the season under the camera, the pace of the weather.
const INTERVAL: f32 = 0.25;

/// Returns the season look the texture takes for a camera `altitude` metres above sea level
/// over the ground `ground` of the planet of `setting` at `season`: the season of that ground,
/// read at the height of the ground and never at the height of the camera, faded out by the
/// height of the camera above it and rounded to the steps of the texture.
pub(super) fn look_over(
    setting: &Setting,
    ground: &Spot,
    altitude: f32,
    season: f32,
) -> SeasonLook {
    SeasonLook::at(setting, ground, season)
        .seen(altitude - ground.altitude)
        .steps()
}

/// Where the camera is and when, for the season.
#[derive(SystemParam)]
struct Here<'w> {
    vantage: Res<'w, Vantage>,
    local: Res<'w, LocalSystem>,
    clock: Res<'w, Clock>,
    planet: Option<Res<'w, Planet>>,
}

impl Here<'_> {
    /// Returns the season look of the ground under the camera, or nothing outside a world.
    fn look(&self) -> Option<SeasonLook> {
        let vantage = &*self.vantage;
        let near = self.local.0.as_ref()?;
        let spec = near.body(u8::try_from(vantage.body).ok()?, vantage.moon)?;
        let observer = &vantage.observer;
        let dir =
            (observer.spin.inverse() * (observer.eye - observer.center)).normalize_or(DVec3::Y);
        let streamed = self
            .planet
            .as_deref()
            .filter(|planet| planet.body == spec.id);
        let sea = Spot {
            dir,
            altitude: 0.0,
            inland: 0.0,
            slope: Vec2::ZERO,
        };
        let spot = streamed.map_or(sea, |planet| planet.globe.spot(dir));
        let season = crate::climate::season(spec, self.clock.seconds);
        Some(look_over(
            &Setting::of(spec),
            &spot,
            vantage.altitude,
            season,
        ))
    }
}

/// Repaints the lookup texture when the season look under the camera changes, and back to the
/// colours without seasons outside a world. The ground is read every [`INTERVAL`] seconds, and
/// at once for a texture that shows nothing yet. The texture
/// keeps its size, so the GPU copy is written over in place and the material stays bound to it.
fn paint(
    time: Res<Time>,
    mut wait: Local<f32>,
    here: Here,
    mut lut: ResMut<Lut>,
    mut images: ResMut<Assets<Image>>,
) {
    *wait -= time.delta_secs();
    if *wait > 0.0 && lut.shown.is_some() {
        return;
    }
    *wait = INTERVAL;
    let look = here.look().unwrap_or(SeasonLook::NONE);
    let Lut {
        image,
        original,
        shown,
        ..
    } = &mut *lut;
    if *shown == Some(look) {
        return;
    }
    let Some(mut image) = image.as_ref().and_then(|handle| images.get_mut(handle)) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    if data.len() != original.len() {
        return;
    }
    repaint(original, data, look);
    *shown = Some(look);
}

/// Registers the lookup texture and its repaint.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Lut>()
        .add_systems(Update, (find, paint).chain());
}
