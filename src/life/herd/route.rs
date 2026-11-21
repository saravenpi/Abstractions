//! Routes: where a herd is at a universe time, from its waypoints, as a pure function of time.
//!
//! Time is cut into legs of one period, shifted by a phase of the herd. Leg `k` ends at waypoint `k`,
//! drawn from `hash(herd id, k)` inside the home range. A leg starts with the eased walk from waypoint
//! `k - 1` to waypoint `k`, then the herd stays there until the next leg. A walk always fits in a
//! period, so the herd is at waypoint `k` when leg `k + 1` starts, and its path is continuous. A
//! waypoint is drawn only where the straight way to it from the home is walkable for the body of
//! the species (see `path`), and a walk too steep to take straight turns through the home.

use bevy::math::{DVec2, DVec3};

use super::HerdState;
use super::path::{Path, walkable};
use super::roll::Roam;
use super::scene::Scene;
use super::sphere::{bearing, east};
use crate::life::body::Locomotion;
use crate::life::body::slope;
use crate::life::census::{self, Site};
use crate::life::{Activity, Habitat, Species};
use crate::universe::hash::{Rng, child};
use crate::world::globe::Tangent;

/// Seconds a herd stays at each waypoint, at the least.
const DWELL: f64 = 240.0;

/// Draws of a waypoint before the herd falls back to a place known to suit it.
const DRAWS: u64 = 4;

/// Height in metres above sea level under which a ground herd at a waypoint drinks.
const DRINK: f32 = 3.0;

/// Salt of the phase of a herd.
const PHASE: u64 = 0x5048_4153;

/// Where a herd is and what it does at one time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::life) struct Pose {
    /// Unit direction of its centre.
    pub centre: DVec3,
    /// Unit tangent it faces.
    pub heading: DVec3,
    /// Speed of its centre in metres per second.
    pub speed: f32,
    /// What it does.
    pub state: HerdState,
}

/// Returns the walking speed in metres per second of `species`: an eighth of its top speed.
fn cruise(species: &Species) -> f64 {
    (f64::from(species.history.speed) * 0.125).max(0.3)
}

/// Draws waypoint `leg` of `roam` of `species` on a planet of `radius` metres, read by `sample` and
/// `height`, and returns it with its site.
pub(super) fn draw(
    roam: &Roam,
    species: &Species,
    leg: i64,
    radius: f64,
    (sample, height): (&dyn Fn(DVec3) -> Site, &dyn Fn(DVec3) -> f32),
) -> (DVec3, Site) {
    let chart = Tangent::new(roam.home, east(roam.home), radius, DVec2::ZERO);
    let mut rng = Rng::new(child(roam.id, leg as u64));
    for _ in 0..DRAWS {
        let reach = roam.reach * rng.unit().sqrt();
        let (sin, cos) = (rng.unit() * std::f64::consts::TAU).sin_cos();
        let dir = chart.dir(DVec2::new(cos, sin) * reach);
        if !walkable(height, (roam.home, dir), radius, footing(species)) {
            continue;
        }
        let site = sample(dir);
        if census::density(species, &site) > 0.0 {
            return (dir, site);
        }
    }
    (roam.home, sample(roam.home))
}

/// Returns the lowest ground in metres above sea level the body of `species` walks on, the deepest
/// water it wades: none for swimmers and flyers, its legs for the shore, sea level for the land. The
/// herds keep to it far from the player, and the drawn animals near.
pub fn floor(species: &Species) -> f32 {
    let free = matches!(
        species.body.locomotion,
        Locomotion::Swim | Locomotion::Fly | Locomotion::Swarm
    );
    match species.role.habitat {
        Habitat::Water => f32::NEG_INFINITY,
        _ if free => f32::NEG_INFINITY,
        Habitat::Shore => -species.body.height.max(0.1),
        Habitat::Ground | Habitat::Air => 0.0,
    }
}

/// Returns the steepest slope in radians the body of `species` walks, and the lowest ground in metres
/// it walks on, see [`floor`].
pub(in crate::life) fn footing(species: &Species) -> (f32, f32) {
    (slope(&species.body), floor(species))
}

/// Returns the way `roam` of `species` walks leg `leg` from waypoint `from` to waypoint `to`, read by
/// `height` on a planet of `radius` metres: straight when walkable, else through its home.
pub(super) fn way(
    roam: &Roam,
    species: &Species,
    (from, to): (DVec3, DVec3),
    (height, radius): (&dyn Fn(DVec3) -> f32, f64),
) -> Path {
    let straight = walkable(height, (from, to), radius, footing(species));
    Path {
        from,
        via: (!straight).then_some(roam.home),
        to,
    }
}

/// Returns where `roam` of `species` is on its route in `scene` at `time`.
pub(super) fn pose(scene: &Scene, roam: &Roam, species: &Species, time: f64) -> Pose {
    let walk = cruise(species);
    let period = 2.0 * roam.reach / walk + DWELL;
    let x = time / period + Rng::new(child(roam.id, PHASE)).unit();
    let leg = x.floor();
    let into = (x - leg) * period;
    let (from, _) = scene.waypoint(roam, species, leg as i64 - 1);
    let (to, site) = scene.waypoint(roam, species, leg as i64);
    let path = scene.path(roam, species, leg as i64, (from, to));
    let span = path.length(scene.planet);
    let walking = span / walk;
    if into >= walking {
        let light = (scene.ground.light)(to, time);
        let last = path.via.unwrap_or(from);
        return Pose {
            centre: to,
            heading: bearing(to, last, to),
            speed: 0.0,
            state: settle(species, &site, light),
        };
    }
    let u = into / walking;
    let (centre, heading) = path.at(span * u * u * (3.0 - 2.0 * u), scene.planet);
    Pose {
        centre,
        heading,
        speed: (span * 6.0 * u * (1.0 - u) / walking) as f32,
        state: HerdState::Travel,
    }
}

/// Returns whether a species out at `activity` is active under `light`, and whether it is deep in
/// its inactive hours.
pub(in crate::life) fn open(activity: Activity, light: f32) -> (bool, bool) {
    match activity {
        Activity::Day => (light >= 0.25, light < 0.1),
        Activity::Night => (light <= 0.25, light > 0.6),
        Activity::Twilight => ((0.05..=0.6).contains(&light), light < 0.05),
    }
}

/// Returns what a herd of `species` does while it stays at a waypoint with `site` under `light`.
fn settle(species: &Species, site: &Site, light: f32) -> HerdState {
    let ground = matches!(species.role.habitat, Habitat::Ground | Habitat::Shore);
    match open(species.role.activity, light) {
        (false, true) => HerdState::Sleep,
        (false, false) => HerdState::Rest,
        _ if ground && site.height < DRINK => HerdState::Drink,
        _ => HerdState::Graze,
    }
}
