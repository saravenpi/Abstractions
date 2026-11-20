//! The light of the galaxies: one cube map of the starlight of every galaxy in sight, baked off
//! the main thread. Each line of sight is integrated through the real star density of the galaxy
//! it crosses, with the dust of that galaxy absorbing, so the band of the home galaxy, the lanes
//! in it and the galaxies far outside it are one model. The sky shader reads the map.

mod march;
mod pack;
#[cfg(test)]
mod rig;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_sight;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};
use bevy::tasks::{
    AsyncComputeTaskPool, Task, TaskPool, block_on, futures_lite::future::poll_once,
};

use self::march::{BYTES, FACES, SIDE, Source, bake};
use super::vantage::{LocalSystem, Vantage};
use crate::screens::Screen;
use crate::space::UniPos;
use crate::universe::{GLY, GalaxyId, GalaxySpec, Glow, Universe};

/// Galaxy grid cells searched for galaxies around the camera's own.
const CELLS: i32 = 2;

/// Metres the camera moves inside a galaxy before the map is baked again.
const INSIDE: f64 = 50.0 * GLY;

/// Share of the distance to the nearest galaxy centre the camera moves outside every galaxy
/// before the map is baked again.
const OUTSIDE: f64 = 0.02;

/// The place a map was baked for, and how far the camera may move from it before it is stale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Built {
    /// Galaxy the camera was in.
    galaxy: GalaxyId,
    /// Position of the camera in metres from the centre of that galaxy.
    at: DVec3,
    /// Metres the camera may move from `at` before the map is baked again.
    limit: f64,
}

impl Built {
    /// Returns true when a camera at `at` metres from the centre of `galaxy` needs a new map.
    pub fn stale(&self, galaxy: GalaxyId, at: DVec3) -> bool {
        self.galaxy != galaxy || self.at.distance(at) > self.limit
    }
}

/// The map of the light of the galaxies, and the bake that is making the next one.
#[derive(Resource, Default)]
pub struct GalaxyLight {
    /// The cube texture the sky reads.
    image: Handle<Image>,
    /// The place the texture was last baked for, once it holds a map.
    built: Option<Built>,
    /// The bake in progress.
    task: Option<Task<(Built, Vec<u8>)>>,
}

impl GalaxyLight {
    /// Returns the texture once it holds the light of the sky, and nothing before.
    pub fn map(&self) -> Option<Handle<Image>> {
        self.built.map(|_| self.image.clone())
    }
}

/// Returns the galaxies in sight of a camera `at` metres from the centre of `galaxy`, and the
/// metres the camera may move before they are baked again: [`INSIDE`] in a galaxy, and
/// [`OUTSIDE`] of the distance to the nearest galaxy centre elsewhere.
pub fn plan(universe: &Universe, galaxy: GalaxyId, at: DVec3) -> (Vec<Source>, f64) {
    let Some(here) = universe.galaxy(galaxy).map(|spec| spec.locate(at)) else {
        return (Vec::new(), INSIDE);
    };
    let source = |spec: &GalaxySpec| {
        let offset = UniPos::new(spec.id.0, spec.center) - here;
        Source {
            glow: Glow::of(spec),
            offset,
            inside: offset.length() <= spec.radius,
        }
    };
    let sources: Vec<Source> = (universe.galaxies_near(galaxy, CELLS).iter())
        .map(source)
        .collect();
    let nearest = (sources.iter()).map(|source| source.offset.length());
    let nearest = nearest.fold(f64::MAX, f64::min);
    let inside = sources.iter().any(|source| source.inside);
    (sources, if inside { INSIDE } else { OUTSIDE * nearest })
}

/// Returns the empty cube texture of the map: six faces of [`SIDE`] texels, read smoothly.
fn blank() -> Image {
    let side = SIDE as u32;
    let mut image = Image::new(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: FACES as u32,
        },
        TextureDimension::D2,
        vec![0; BYTES],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image.sampler = ImageSampler::linear();
    image
}

/// Starts a fresh map when the world starts loading.
fn start(mut light: ResMut<GalaxyLight>, mut images: ResMut<Assets<Image>>) {
    *light = GalaxyLight {
        image: images.add(blank()),
        ..default()
    };
}

/// Writes `bytes` over the texels of the texture `image`, which keeps its size.
fn write(images: &mut Assets<Image>, image: &Handle<Image>, bytes: &[u8]) {
    let Some(mut image) = images.get_mut(image) else {
        return;
    };
    if let Some(data) = image.data.as_mut().filter(|data| data.len() == bytes.len()) {
        data.copy_from_slice(bytes);
    }
}

/// Takes a finished bake into the texture, and starts the next one when the camera has moved
/// far enough from the place of the last, unless one is already running. Waits for the star
/// system of the world to be known, so a stale place is never baked for.
fn keep(
    mut light: ResMut<GalaxyLight>,
    (vantage, local): (Res<Vantage>, Res<LocalSystem>),
    universe: Option<Res<Universe>>,
    mut images: ResMut<Assets<Image>>,
) {
    let done = light
        .task
        .as_mut()
        .and_then(|task| block_on(poll_once(task)));
    if let Some((built, bytes)) = done {
        write(&mut images, &light.image, &bytes);
        (light.built, light.task) = (Some(built), None);
    }
    let (Some(universe), Some(_)) = (universe, local.0.as_ref()) else {
        return;
    };
    let (galaxy, at) = (vantage.galaxy, vantage.galactic);
    let fresh = light.built.is_some_and(|built| !built.stale(galaxy, at));
    if light.task.is_some() || fresh {
        return;
    }
    let universe = *universe;
    let pool = AsyncComputeTaskPool::get_or_init(TaskPool::default);
    light.task = Some(pool.spawn(async move {
        let (sources, limit) = plan(&universe, galaxy, at);
        (Built { galaxy, at, limit }, bake(&sources))
    }));
}

/// Registers the map of the light of the galaxies and the system that keeps it.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GalaxyLight>()
        .add_systems(OnEnter(Screen::Loading), start)
        .add_systems(PostUpdate, keep.after(super::sun::track));
}
