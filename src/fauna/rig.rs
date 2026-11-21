//! The test rig of the drawn animals: a real living home planet, the robot standing in a herd, and
//! the scan, the walk and the fades stepped by hand.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::herds::{Land, read};
use super::stage::stage;
use super::want::drawn;
use crate::life::body::Locomotion;
use crate::life::herd::{Herd, Search, Watch, near};
use crate::life::hunt::{Hunt, Outcome};
use crate::life::{self, Life, Species};
use crate::space::Anchor;
use crate::universe::{BodyId, Universe};
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Universe seed of the rig: its home planet is homely, and the first herd of walkers its life gives
/// near the rig's spots is of plant eaters a robot outweighs, which flee it.
pub const SEED: u64 = 3;

/// Universe time in seconds the rig starts at.
pub const START: f64 = 600.0;

/// Returns the home planet of the rig, its globe carrying `alive` life or none, and that life.
pub fn planet(alive: bool) -> (BodyId, Arc<Globe>, Option<Life>) {
    let universe = Universe::new(SEED);
    let body = universe.home();
    let spec = universe.planet(body);
    let star = universe.system(body.star).star;
    let life = life::of(&spec, &star).filter(|_| alive);
    let globe = Globe::of(&spec).with_life(life.clone());
    (body, Arc::new(globe), life)
}

/// Returns the first herd of a drawn walking species that `fits` near one of a few spots of the
/// planet of `globe`.
pub fn herd_of(
    globe: &Arc<Globe>,
    life: &Life,
    body: BodyId,
    fits: impl Fn(&Species) -> bool,
) -> Option<Herd> {
    let spec = Universe::new(SEED).planet(body);
    let land = Land {
        globe,
        life,
        planet: &spec,
    };
    let spots = (0..24).map(|i| {
        let a = f64::from(i) * 0.7;
        DVec3::new(a.cos(), 0.3 * a.sin(), a.sin()).normalize()
    });
    spots.into_iter().find_map(|at| {
        let search = Search {
            at,
            radius: 2000.0,
            planet: globe.radius(),
            time: START,
        };
        let herds = read(land, START, |g| near(life, g, &search));
        let walks = |s: &Species| drawn(s) && s.body.locomotion == Locomotion::Walk && fits(s);
        let walker =
            |h: &Herd| life.species.get(usize::from(h.species)).is_some_and(walks) && h.count > 1;
        herds.into_iter().find(walker)
    })
}

/// Returns the terrain view and the anchor around unit direction `dir` of `globe`, and the render
/// position of the ground there.
pub fn view(globe: &Arc<Globe>, body: BodyId, dir: DVec3) -> (Terrain, Anchor, Vec3) {
    let chart = globe.face_chart(Face::locate(dir).0);
    let at = chart.locate(dir);
    let terrain = Terrain::on(globe.clone(), chart).around(at.x, at.y);
    let ground = Vec3::new(at.x, terrain.mesh_height(at.x, at.y), at.y);
    (
        terrain.clone(),
        Anchor::surface(body, &terrain, ground),
        ground,
    )
}

/// Returns an app on the rig's home planet, `alive` or dead, with the robot standing at the centre
/// of a herd, and that herd when the planet has one.
pub fn app(alive: bool) -> (App, Option<Herd>) {
    app_of(alive, |_| true)
}

/// Returns an app like [`app`] with the robot in the first herd of a species that `fits`.
pub fn app_of(alive: bool, fits: impl Fn(&Species) -> bool) -> (App, Option<Herd>) {
    let (body, globe, life) = planet(true);
    let herd = life.as_ref().and_then(|l| herd_of(&globe, l, body, &fits));
    let dir = herd.map_or(DVec3::Y, |h| h.centre);
    let globe = if alive { globe } else { planet(false).1 };
    (stage(globe, body, dir, START), herd.filter(|_| alive))
}

/// Returns the first kill of a hunt around a few spots of the planet of `globe` within ten minutes
/// after [`START`].
pub fn kill(globe: &Arc<Globe>, life: &Life, body: BodyId) -> Option<Hunt> {
    let spec = Universe::new(SEED).planet(body);
    let land = Land {
        globe,
        life,
        planet: &spec,
    };
    let spots = (0..8).map(|i| DVec3::new(f64::from(i).cos(), 0.2, f64::from(i).sin()).normalize());
    let windows = spots.flat_map(|at| (0..5).map(move |k| (at, START + 110.0 * f64::from(k))));
    windows.into_iter().find_map(|(at, time)| {
        let search = Search {
            at,
            radius: 2000.0,
            planet: globe.radius(),
            time,
        };
        let watch = read(land, time, |g| Watch::new(life, g, &search));
        let times = (0..12).map(|k| time + 10.0 * f64::from(k));
        let mut kills = times.flat_map(|t| watch.hunts(life, t));
        kills.find(|h| matches!(h.outcome, Outcome::Kill { .. }) && h.outcome_time() > time + 20.0)
    })
}
