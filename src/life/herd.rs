//! Herds: groups of animals that are pure functions of the life of a planet, its seed and universe time.
//!
//! Every player computes the same herds at the same time with no network. The sphere is cut into
//! cells of about 256 m, and of about 1 km for hunters (see `cells`); each cell holds herds rolled
//! from the census density at its centre times its area. A herd has a home in its cell and walks a
//! route of waypoints inside its home range (see `route`). Hunts (`crate::life::hunt`) bend the
//! route of hunters and take members from their prey. Herd centres are unit directions; members are
//! offsets in metres in the tangent plane, x east and y north, north being the spin axis, world +Y.

mod cells;
#[cfg(test)]
pub(super) mod fixtures;
mod roll;
mod route;
mod scene;
mod sphere;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cost;

use bevy::math::{DVec3, Vec2};

use super::census::Site;
use super::hunt::{self, Hunt};
use super::{Life, Species};
use crate::universe::hash::{Rng, child};
pub(super) use roll::{Roam, hunts};
pub(super) use route::{Pose, open};
pub(super) use scene::Scene;
#[cfg(test)]
pub(super) use sphere::arc;
pub(super) use sphere::{bearing, east, slerp};

/// Most deaths a herd remembers at once.
pub const DEATHS: usize = 6;

/// What a herd does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HerdState {
    /// Feeds at a waypoint.
    Graze,
    /// Walks between waypoints.
    Travel,
    /// Drinks at a waypoint by the water.
    Drink,
    /// Stays still at a waypoint outside its active hours.
    Rest,
    /// Sleeps at a waypoint deep in its inactive hours.
    Sleep,
    /// Creeps up on prey, or lies in wait for it.
    Stalk,
    /// Runs prey down.
    Chase,
    /// Eats a kill.
    Feed,
}

/// The death of one member: its slot and the universe time it died. It is back after
/// [`hunt::RECOVERY`] seconds, as a new member born into the herd.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Death {
    /// Slot of the member.
    pub slot: u8,
    /// Universe time of the death in seconds.
    pub at: f64,
}

/// One herd at one time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Herd {
    /// Stable identity.
    pub id: u64,
    /// Index of its species in [`Life::species`].
    pub species: u16,
    /// Living members at the search time, young included.
    pub count: u8,
    /// Slots of the herd when no member is dead: slots run from 0 to `base`.
    pub base: u8,
    /// Young among the members: slots below `young`.
    pub young: u8,
    /// Unit direction of its centre.
    pub centre: DVec3,
    /// Unit tangent at the centre it faces.
    pub heading: DVec3,
    /// Speed of the centre in metres per second.
    pub speed: f32,
    /// What it does.
    pub state: HerdState,
    /// Unit direction of the centre of its home range.
    pub home: DVec3,
    /// Usual distance between members in metres, from the size of its species.
    pub spacing: f32,
    /// Recent and coming deaths, earliest first.
    pub deaths: [Option<Death>; DEATHS],
}

/// One member of a herd at one time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Member {
    /// Offset from the herd centre in metres in the tangent plane, x east and y north.
    pub offset: Vec2,
    /// True for the young.
    pub young: bool,
    /// Phase of its gait and idle motion, 0 to 1.
    pub phase: f32,
}

/// Where and when to look for herds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Search {
    /// Unit direction looked around.
    pub at: DVec3,
    /// Distance in metres along the ground within which herd centres are returned.
    pub radius: f64,
    /// Sea level radius of the planet in metres.
    pub planet: f64,
    /// Universe time in seconds.
    pub time: f64,
    /// The robot's wake point on the home planet, as a unit direction, whose herd is always there.
    pub pin: Option<DVec3>,
}

/// What herds read of the planet: the site at a unit direction, and the daylight, 0 to 1, at a unit
/// direction and a universe time.
pub struct Ground<'a> {
    /// Site at a unit direction. Called at most a few hundred times per search.
    pub sample: &'a dyn Fn(DVec3) -> Site,
    /// Daylight at a unit direction and a universe time in seconds. Cheap, called freely.
    pub light: &'a dyn Fn(DVec3, f64) -> f32,
}

/// Returns every herd of `life` with a living member whose centre is within the search radius at the
/// search time, in a stable order.
pub fn near(life: &Life, ground: &Ground, search: &Search) -> Vec<Herd> {
    let scene = Scene::build(life, ground, search);
    let hunts = hunt::plan(
        &scene,
        (search.time - hunt::RECOVERY, search.time),
        (search.at, search.radius),
    );
    scene
        .roams
        .iter()
        .filter(|r| scene.metres(r.home, search.at) <= search.radius + r.reach + scene.side)
        .filter_map(|r| herd(&scene, r, &hunts, search.time))
        .filter(|h| h.count > 0 && scene.metres(h.centre, search.at) <= search.radius)
        .collect()
}

/// Returns `roam` at `time`, with the hunts of `hunts`, or nothing when its species is missing.
fn herd(scene: &Scene, roam: &Roam, hunts: &[Hunt], time: f64) -> Option<Herd> {
    let species = scene.life.species.get(usize::from(roam.species))?;
    let base = scene.pose(roam, time)?;
    let pose = hunts
        .iter()
        .find(|h| h.hunter == roam.id && h.start <= time && time < h.end())
        .map_or(base, |h| hunt::bend(scene, &base, h, time));
    let deaths = hunt::deaths(hunts, roam.id, time);
    let dead = (0..roam.count).filter(|s| dead(&deaths, *s, time)).count() as u8;
    Some(Herd {
        id: roam.id,
        species: roam.species,
        count: roam.count - dead,
        base: roam.count,
        young: roam.young,
        centre: pose.centre,
        heading: pose.heading,
        speed: pose.speed,
        state: pose.state,
        home: roam.home,
        spacing: spacing(species),
        deaths,
    })
}

/// Returns the usual distance in metres between members of a herd of `species`.
pub(super) fn spacing(species: &Species) -> f32 {
    (0.6 * species.history.mass.cbrt()).max(0.1)
}

/// Returns true when `slot` is dead at `time` by one of `deaths`.
fn dead(deaths: &[Option<Death>], slot: u8, time: f64) -> bool {
    deaths
        .iter()
        .flatten()
        .any(|d| d.slot == slot && d.at <= time && time < d.at + hunt::RECOVERY)
}

/// Returns member `slot` of `herd` at universe `time`, or nothing when that slot is empty or dead.
pub fn member(herd: &Herd, slot: u8, time: f64) -> Option<Member> {
    if slot >= herd.base || dead(&herd.deaths, slot, time) {
        return None;
    }
    Some(Member {
        offset: offset(herd.id, herd.spacing, slot, time),
        young: slot < herd.young,
        phase: Rng::new(child(herd.id ^ 0x5048, u64::from(slot))).unit() as f32,
    })
}

/// Returns the offset in metres, x east and y north, of member `slot` of herd `id` at `time`: the
/// young nearest the centre, each member at its own seeded place on a sunflower spiral, drifting
/// slowly around it.
pub(super) fn offset(id: u64, spacing: f32, slot: u8, time: f64) -> Vec2 {
    let mut rng = Rng::new(child(id, u64::from(slot)));
    let rank = f64::from(slot) + 0.5;
    let spacing = f64::from(spacing);
    let angle = rank * 2.399_963 + Rng::new(id).unit() * std::f64::consts::TAU;
    let ring = spacing * rank.sqrt();
    let jitter = (rng.range(-0.35, 0.35), rng.range(-0.35, 0.35));
    let (rate, shift) = (rng.range(0.02, 0.06), rng.range(0.0, std::f64::consts::TAU));
    let drift = 0.3 * spacing;
    let x = ring * angle.cos() + jitter.0 * spacing + drift * (rate * time + shift).sin();
    let y = ring * angle.sin() + jitter.1 * spacing + drift * (rate * 0.8 * time + shift).cos();
    Vec2::new(x as f32, y as f32)
}
