//! The light of the galaxies: one cube map of the starlight of every galaxy in sight, baked off
//! the main thread. Each line of sight is integrated through the real star density of the galaxy
//! it crosses, with the dust of that galaxy absorbing, so the band of the home galaxy, the lanes
//! in it and the galaxies far outside it are one model. A line of sight is cut by one rule for a
//! camera inside a galaxy and outside it, see [`steps`], and a new map fades in over the one on
//! show, see [`fade`]. The sky shader reads the map.

mod fade;
mod fine;
mod image;
mod march;
mod pack;
#[cfg(test)]
mod rig;
mod steps;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_fade;
#[cfg(test)]
mod tests_far;
#[cfg(test)]
mod tests_gap;
#[cfg(test)]
mod tests_light;
#[cfg(test)]
mod tests_sight;
mod tune;

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::tasks::{
    AsyncComputeTaskPool, Task, TaskPool, block_on, futures_lite::future::poll_once,
};

use self::fade::Fade;
use self::image::{blank, write};
use self::march::{Source, bake};
use self::tune::MIN_GAP;
use super::vantage::{LocalSystem, Vantage};
use crate::screens::Screen;
use crate::space::UniPos;
use crate::universe::{GalaxyId, GalaxySpec, Glow, LY, RADIUS_LY, Universe};

/// Galaxy grid cells searched for galaxies around the camera's own.
const CELLS: i32 = 2;

/// Share of the radius of the galaxy the camera is in that it moves before the map is baked
/// again: 1,000 light-years in a galaxy 50,000 across the radius, as [`OUTSIDE`] is a share of a
/// distance.
const INSIDE: f64 = 0.02;

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
    /// The map on show and the fade of it to the last bake.
    fade: Fade,
    /// The bake in progress.
    task: Option<Task<(Built, Vec<u8>)>>,
    /// Seconds of the app clock at which the last bake was taken into the texture.
    finished: Option<f64>,
}

impl GalaxyLight {
    /// Takes a finished bake, if there is one, into the fade, and writes the fade to the texture,
    /// at `now` seconds of the app clock.
    fn land(&mut self, images: &mut Assets<Image>, now: f64) {
        let done = self
            .task
            .as_mut()
            .and_then(|task| block_on(poll_once(task)));
        if let Some((built, bytes)) = done {
            self.fade.begin(bytes, now);
            (self.built, self.task, self.finished) = (Some(built), None, Some(now));
        }
        if let Some(bytes) = self.fade.step(now) {
            write(images, &self.image, bytes);
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
/// metres the camera may move before they are baked again: [`INSIDE`] of the radius of the galaxy
/// it is in, and [`OUTSIDE`] of the distance to the nearest galaxy centre elsewhere.
pub fn plan(universe: &Universe, galaxy: GalaxyId, at: DVec3) -> (Vec<Source>, f64) {
    let Some(here) = universe.galaxy(galaxy).map(|spec| spec.locate(at)) else {
        return (Vec::new(), INSIDE * RADIUS_LY.0 * LY);
    };
    let source = |spec: &GalaxySpec| {
        let offset = UniPos::sectors(spec.id.0, spec.center_sector, DVec3::ZERO) - here;
        Source {
            glow: Glow::of(spec),
            offset,
            inside: offset.length() <= spec.radius,
        }
    };
    let specs = universe.galaxies_near(galaxy, CELLS);
    let sources: Vec<Source> = specs.iter().map(source).collect();
    let nearest = (sources.iter()).map(|source| source.offset.length());
    let nearest = nearest.fold(f64::MAX, f64::min);
    let radius = specs.iter().zip(&sources).find(|(_, source)| source.inside);
    let limit = radius.map_or(OUTSIDE * nearest, |(spec, _)| INSIDE * spec.radius);
    (sources, limit)
}

/// Starts a fresh map when the world starts loading.
fn start(mut light: ResMut<GalaxyLight>, mut images: ResMut<Assets<Image>>) {
    *light = GalaxyLight {
        image: images.add(blank()),
        ..default()
    };
}

/// Takes a finished bake into the fade of the texture, and starts the next one when the camera has moved
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
