//! The light of the galaxies: one cube map of the starlight of every galaxy in sight, baked off
//! the main thread. Each line of sight is integrated through the real star density of the galaxy
//! it crosses, with the dust of that galaxy absorbing, so the band of the home galaxy, the lanes
//! in it and the galaxies far outside it are one model. The sky shader reads the map.

mod image;
mod march;
mod pack;
#[cfg(test)]
mod rig;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_gap;
#[cfg(test)]
mod tests_sight;
mod tune;

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::tasks::{
    AsyncComputeTaskPool, Task, TaskPool, block_on, futures_lite::future::poll_once,
};

use self::image::{blank, write};
use self::march::{Source, bake};
use self::tune::MIN_GAP;
use super::vantage::{LocalSystem, Vantage};
use crate::screens::Screen;
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
    /// Seconds of the app clock at which the last bake was taken into the texture.
    finished: Option<f64>,
}

impl GalaxyLight {
    /// Takes a finished bake, if there is one, into the texture, at `now` seconds of the app clock.
    fn land(&mut self, images: &mut Assets<Image>, now: f64) {
        let done = self
            .task
            .as_mut()
            .and_then(|task| block_on(poll_once(task)));
        if let Some((built, bytes)) = done {
            write(images, &self.image, &bytes);
            (self.built, self.task, self.finished) = (Some(built), None, Some(now));
        }
    }

    /// Returns true once [`MIN_GAP`] seconds have passed since the last bake was taken, at `now`.
    fn rested(&self, now: f64) -> bool {
        self.finished.is_none_or(|at| now - at >= MIN_GAP)
    }

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
        let offset = spec.locate(DVec3::ZERO) - here;
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

/// Starts a fresh map when the world starts loading.
fn start(mut light: ResMut<GalaxyLight>, mut images: ResMut<Assets<Image>>) {
    *light = GalaxyLight {
        image: images.add(blank()),
        ..default()
    };
}

/// Takes a finished bake into the texture, and starts the next one when the camera has moved
/// far enough from the place of the last and the last bake is at least [`MIN_GAP`] seconds old,
/// unless one is already running. Waits for the star system of the world to be known, so a stale
/// place is never baked for.
fn keep(
    mut light: ResMut<GalaxyLight>,
    (vantage, local): (Res<Vantage>, Res<LocalSystem>),
    universe: Option<Res<Universe>>,
    (time, mut images): (Res<Time>, ResMut<Assets<Image>>),
) {
    let now = time.elapsed_secs_f64();
    light.land(&mut images, now);
    let (Some(universe), Some(_)) = (universe, local.0.as_ref()) else {
        return;
    };
    let (galaxy, at) = (vantage.galaxy, vantage.galactic);
    let fresh = light.built.is_some_and(|built| !built.stale(galaxy, at));
    if light.task.is_some() || fresh || !light.rested(now) {
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
