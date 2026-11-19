//! Building the starfield off the main thread: gathering the stars reads hundreds of sectors,
//! so it runs in a task and the mesh is swapped in when it is ready. A warp never waits for it.

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future::poll_once;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on};

use super::depth::{Sighted, lit};
use super::{GALAXY_CELLS, MOST_STARS, REACH_GLY, Starfield};
use crate::render::vantage::Scene;
use crate::space::UniPos;
use crate::universe::{GLY, GalaxyId, SECTOR_GLY, Universe};

/// The mesh being built for a starfield, and the place it is built for.
#[derive(Component)]
pub struct Building {
    /// Galaxy and position in metres from its centre the mesh will show the sky of.
    place: (GalaxyId, DVec3),
    /// The build, which gives no mesh when no speck is in sight.
    task: Task<Option<Mesh>>,
}

/// Returns every speck seen from `at` metres from the centre of `galaxy`: the stars within
/// reach and the galaxies around. The star of the current system is one of them: from afar it
/// is this point, and up close its disc is drawn on it as a body.
pub fn gather(universe: &Universe, galaxy: GalaxyId, at: DVec3) -> Vec<Sighted> {
    let stars = universe.stars_near(galaxy, at, REACH_GLY);
    let mut specks: Vec<Sighted> = stars
        .iter()
        .take(MOST_STARS)
        .map(|star| Sighted::star(star, at))
        .collect();
    let Some(here) = universe.galaxy(galaxy).map(|spec| spec.locate(at)) else {
        return specks;
    };
    let others = universe.galaxies_near(galaxy, GALAXY_CELLS);
    let seen = |other: &_| Sighted::galaxy(other, UniPos::new(other.id.0, other.center) - here);
    specks.extend(others.iter().filter_map(seen));
    specks
}

/// Starts a build when the camera has moved more than a sector since the last one, unless one
/// is already running: the running build finishes first, and the next one starts from where
/// the camera is then.
///
/// Waits for the star system of the world to be known, so a stale place is never built for.
pub fn launch(
    mut commands: Commands,
    scene: Scene,
    universe: Option<Res<Universe>>,
    fields: Query<(Entity, &Starfield), Without<Building>>,
) {
    let (Some(universe), Some(_)) = (universe, scene.local.0.as_ref()) else {
        return;
    };
    let vantage = &scene.vantage;
    let (galaxy, at) = (vantage.galaxy, vantage.galactic);
    let near =
        |(built, was): (GalaxyId, DVec3)| built == galaxy && was.distance(at) < SECTOR_GLY * GLY;
    let universe = *universe;
    for (entity, field) in &fields {
        if field.built.is_some_and(near) {
            continue;
        }
        let task = AsyncComputeTaskPool::get().spawn(async move {
            let specks = gather(&universe, galaxy, at);
            (!specks.is_empty()).then(|| lit(&specks))
        });
        let place = (galaxy, at);
        commands.entity(entity).try_insert(Building { place, task });
    }
}

/// Swaps in the mesh of a finished build, or hides the starfield when it found nothing.
pub fn adopt(
    mut commands: Commands,
    mut fields: Query<(Entity, &mut Starfield, &mut Visibility, &mut Building)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut field, mut visibility, mut building) in &mut fields {
        let Some(built) = block_on(poll_once(&mut building.task)) else {
            continue;
        };
        field.built = Some(building.place);
        let mut entity = commands.entity(entity);
        entity.try_remove::<Building>();
        match built {
            Some(mesh) => {
                *visibility = Visibility::Visible;
                entity.try_insert(Mesh3d(meshes.add(mesh)));
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}
