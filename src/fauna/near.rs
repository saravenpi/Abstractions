//! The herds the compass, the minimap and the map mark: only herds the local robot has seen, each where
//! its animals are now, fading out [`MEMORY`] seconds after it was last seen. No mark leads to a herd the
//! player has not found, nor to a place its animals have left.
//!
//! The robot sees a herd when the middle of its drawn animals is within [`SIGHT`] of its eyes, within
//! [`VIEW`] of where it looks, and no ground stands between them. A mark comes from the animals drawn, and
//! once they are not drawn from the centre of the herd they are placed around when drawn again.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::{MARK_APART, MARK_SIZE, MARKS, MEMORY, SIGHT, VIEW};
use super::want::drawn;
use crate::life::body::Locomotion;
use crate::life::herd::Herd;
use crate::life::{Life, Species, SpeciesId};
use crate::universe::BodyId;
use crate::world::Globe;

/// Height samples along a line of sight.
const LOOKS: u32 = 16;

/// One herd the robot has seen.
#[derive(Debug, Clone, PartialEq)]
pub struct Sighting {
    /// Identity of the herd.
    pub id: u64,
    /// Its centre on the ground where it was last seen, in metres from the planet centre in its
    /// turning frame.
    pub at: DVec3,
    /// Its species.
    pub species: SpeciesId,
    /// The name of its species.
    pub name: String,
    /// Metres from the robot to the place it was last seen, at the last scan.
    pub metres: f64,
    /// Universe time in seconds it was last seen.
    pub seen: f64,
}

impl Sighting {
    /// Returns how much of the mark is left at universe `time`: 1 when just seen, 0 after
    /// [`MEMORY`] seconds.
    pub fn fresh(&self, time: f64) -> f32 {
        (1.0 - (time - self.seen) / MEMORY).clamp(0.0, 1.0) as f32
    }
}

/// The herds the compass and the minimap mark, most recently seen first, and the planet they are on.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Nearby {
    /// The planet the herds are on, none off a living planet.
    pub body: Option<BodyId>,
    /// The herds seen within the last [`MEMORY`] seconds, at most [`MARKS`].
    pub herds: Vec<Sighting>,
}

/// Where the robot looks from: its eyes in metres from the planet centre and the unit direction it
/// looks along, both in the turning frame of the planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eye {
    /// Planet position of the eyes.
    pub at: DVec3,
    /// Unit direction looked along.
    pub ahead: DVec3,
}

/// Returns true when animals of `species` are worth a mark: drawn, not a swarm, and at least
/// [`MARK_SIZE`] long, so a mark leads to animals the player can see.
fn markable(species: &Species) -> bool {
    drawn(species)
        && species.body.locomotion != Locomotion::Swarm
        && species.body.length >= MARK_SIZE
}

/// Returns true when `eye` sees planet position `target` on `globe`: near enough, in view, and with
/// no ground between them.
pub fn sees(globe: &Globe, eye: Eye, target: DVec3) -> bool {
    let to = target - eye.at;
    if to.length() > SIGHT || to.angle_between(eye.ahead) > VIEW {
        return false;
    }
    (1..LOOKS).all(|i| {
        let p = eye.at + to * (f64::from(i) / f64::from(LOOKS));
        let dir = p.normalize();
        p.length() >= globe.radius() + f64::from(globe.height(dir).max(0.0))
    })
}

/// Returns where the drawn living members of herd `id` stand on the ground, on average, from `drawn`,
/// the planet position of every such member and its herd: nothing when none of it is drawn.
fn drawn_at(drawn: &[(u64, DVec3)], id: u64) -> Option<DVec3> {
    let (sum, count) = drawn
        .iter()
        .filter(|(herd, _)| *herd == id)
        .fold((DVec3::ZERO, 0u32), |(sum, n), (_, at)| (sum + *at, n + 1));
    (count > 0).then(|| sum / f64::from(count))
}

/// Returns the sighting of `herd` of `life` on `globe` at universe `time` when `eye` sees its drawn
/// members, which stand around `at` on average.
fn sighted(
    life: &Life,
    globe: &Globe,
    (herd, at): (&Herd, DVec3),
    (eye, time): (Eye, f64),
) -> Option<Sighting> {
    let species = life
        .species
        .get(usize::from(herd.species))
        .filter(|s| markable(s))?;
    let body = at + at.normalize() * f64::from(species.body.height.max(0.3));
    sees(globe, eye, body).then(|| Sighting {
        id: herd.id,
        at,
        species: species.id,
        name: species.name.clone(),
        metres: 0.0,
        seen: time,
    })
}

/// Moves `sighting` to where its herd is now: among its drawn members, or else at the centre of the herd
/// in `herds` on `globe`, around which its animals are placed when they are drawn. Returns false when the
/// herd is in neither: every member died, or it went out of the reach of the herds read.
fn follow(
    sighting: &mut Sighting,
    globe: &Globe,
    (herds, drawn): (&[Herd], &[(u64, DVec3)]),
) -> bool {
    let simulated = || {
        let herd = herds.iter().find(|h| h.id == sighting.id)?;
        Some(herd.centre * (globe.radius() + f64::from(globe.height(herd.centre).max(0.0))))
    };
    match drawn_at(drawn, sighting.id).or_else(simulated) {
        Some(at) => {
            sighting.at = at;
            true
        }
        None => false,
    }
}

/// Updates `nearby` for the robot looking from `eye` at universe `time` over `herds` of `life` on
/// `globe`, the planet `body`, `drawn` holding the planet position and the herd of every drawn living
/// member. A herd is marked once the robot sees its drawn animals, at the middle of them; every mark then
/// follows its herd, among its drawn animals or at the centre of the herd they are placed around, until
/// [`MEMORY`] runs out after it was last seen; a herd that died or went out of reach loses its mark. At
/// most [`MARKS`] stay, most recently seen first, each over [`MARK_APART`] from a fresher one so that one
/// mark stands for herds that keep together.
pub fn remember(
    nearby: &mut Nearby,
    (life, globe, body): (&Life, &Globe, Option<BodyId>),
    (herds, drawn): (&[Herd], &[(u64, DVec3)]),
    (eye, time): (Eye, f64),
) {
    if nearby.body != body {
        *nearby = Nearby {
            body,
            herds: Vec::new(),
        };
    }
    let seen = herds
        .iter()
        .filter_map(|h| Some((h, drawn_at(drawn, h.id)?)))
        .filter_map(|h| sighted(life, globe, h, (eye, time)));
    for seen in seen {
        nearby.herds.retain(|s| s.id != seen.id);
        nearby.herds.push(seen);
    }
    nearby
        .herds
        .retain_mut(|s| s.fresh(time) > 0.0 && follow(s, globe, (herds, drawn)));
    keep(nearby, globe, eye);
}

/// Keeps the [`MARKS`] freshest marks of `nearby` at least [`MARK_APART`] apart, with their distances
/// along the ground of `globe` from `eye`.
fn keep(nearby: &mut Nearby, globe: &Globe, eye: Eye) {
    nearby.herds.sort_by(|a, b| b.seen.total_cmp(&a.seen));
    let ground = eye.at.normalize();
    let mut kept: Vec<Sighting> = Vec::with_capacity(MARKS);
    for mut s in std::mem::take(&mut nearby.herds) {
        if kept.len() < MARKS && kept.iter().all(|k| k.at.distance(s.at) > MARK_APART) {
            s.metres = s.at.normalize().angle_between(ground) * globe.radius();
            kept.push(s);
        }
    }
    nearby.herds = kept;
}
