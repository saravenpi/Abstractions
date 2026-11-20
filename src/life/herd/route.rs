//! Routes: where a herd is at a universe time, from its waypoints, as a pure function of time.
//!
//! Time is cut into legs of one period, shifted by a phase of the herd. Leg `k` ends at waypoint `k`,
//! drawn from `hash(herd id, k)` inside the home range. A leg starts with the eased walk from waypoint
//! `k - 1` to waypoint `k`, then the herd stays there until the next leg. A walk always fits in a
//! period, so the herd is at waypoint `k` when leg `k + 1` starts, and its path is continuous.

use bevy::math::{DVec2, DVec3};

use super::HerdState;
use super::roll::Roam;
use super::scene::Scene;
use super::sphere::{bearing, east, slerp};
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

/// Draws waypoint `leg` of `roam` of `species` on a planet of `radius` metres, and returns it with its site.
pub(super) fn draw(
    roam: &Roam,
    species: &Species,
    leg: i64,
    radius: f64,
    sample: &dyn Fn(DVec3) -> Site,
) -> (DVec3, Site) {
    let chart = Tangent::new(roam.home, east(roam.home), radius, DVec2::ZERO);
    let mut rng = Rng::new(child(roam.id, leg as u64));
    for _ in 0..DRAWS {
        let reach = roam.reach * rng.unit().sqrt();
        let (sin, cos) = (rng.unit() * std::f64::consts::TAU).sin_cos();
        let dir = chart.dir(DVec2::new(cos, sin) * reach);
        let site = sample(dir);
        if census::density(species, &site) > 0.0 {
            return (dir, site);
        }
    }
    (roam.fallback, sample(roam.fallback))
}

/// Returns where `roam` of `species` is on its route in `scene` at `time`.
pub(super) fn pose(scene: &Scene, roam: &Roam, species: &Species, time: f64) -> Pose {
    let walk = cruise(species);
    let slack = roam.reach + roam.home.distance(roam.fallback) * scene.planet;
    let period = 2.0 * slack / walk + DWELL;
    let x = time / period + Rng::new(child(roam.id, PHASE)).unit();
    let leg = x.floor();
    let into = (x - leg) * period;
    let (from, _) = scene.waypoint(roam, species, leg as i64 - 1);
    let (to, site) = scene.waypoint(roam, species, leg as i64);
    let span = scene.metres(from, to);
    let walking = span / walk;
    if into >= walking {
        let light = (scene.ground.light)(to, time);
        return Pose {
            centre: to,
            heading: bearing(to, from, to),
            speed: 0.0,
            state: settle(species, &site, light),
        };
    }
    let u = into / walking;
    let centre = slerp(from, to, u * u * (3.0 - 2.0 * u));
    Pose {
        centre,
        heading: bearing(centre, from, to),
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
