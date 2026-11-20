//! The herds near the local robot that the compass and the minimap mark: the nearest few within
//! [`MARK_REACH`], rewritten at every scan.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::herds::herd_arc;
use super::tune::{MARK_APART, MARK_REACH, MARK_SIZE, MARKS};
use super::want::drawn;
use crate::life::body::Locomotion;
use crate::life::herd::Herd;
use crate::life::{Life, Species, SpeciesId};
use crate::universe::BodyId;
use crate::world::Globe;

/// One herd near the robot.
#[derive(Debug, Clone, PartialEq)]
pub struct Sighting {
    /// Its centre on the ground, in metres from the planet centre in its turning frame.
    pub at: DVec3,
    /// Its species.
    pub species: SpeciesId,
    /// The name of its species.
    pub name: String,
    /// Metres along the ground from the robot at the last scan.
    pub metres: f64,
}

/// The herds the compass and the minimap mark, nearest first, and the planet they are on.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Nearby {
    /// The planet the herds are on, none off a living planet.
    pub body: Option<BodyId>,
    /// The nearest herds within [`MARK_REACH`], at most [`MARKS`].
    pub herds: Vec<Sighting>,
}

/// Returns true when animals of `species` are worth a mark: drawn, not a swarm, and at least
/// [`MARK_SIZE`] long, so a mark leads to animals the player can see.
fn seen(species: &Species) -> bool {
    drawn(species)
        && species.body.locomotion != Locomotion::Swarm
        && species.body.length >= MARK_SIZE
}

/// Returns the herds of `herds` of `life` on `globe` worth marking for a robot at unit direction
/// `robot`: those [`seen`] within [`MARK_REACH`], nearest first, at most [`MARKS`], each
/// over [`MARK_APART`] from a nearer one so that one mark stands for herds that keep together.
pub fn sightings(life: &Life, globe: &Globe, herds: &[Herd], robot: DVec3) -> Vec<Sighting> {
    let radius = globe.radius();
    let mut out: Vec<Sighting> = herds
        .iter()
        .filter_map(|h| {
            let species = life
                .species
                .get(usize::from(h.species))
                .filter(|s| seen(s))?;
            let metres = herd_arc(h.centre, robot) * radius;
            let lift = f64::from(globe.height(h.centre).max(0.0));
            (metres <= MARK_REACH).then(|| Sighting {
                at: h.centre * (radius + lift),
                species: species.id,
                name: species.name.clone(),
                metres,
            })
        })
        .collect();
    out.sort_by(|a, b| a.metres.total_cmp(&b.metres));
    let mut kept: Vec<Sighting> = Vec::with_capacity(MARKS);
    for s in out {
        if kept.len() < MARKS && kept.iter().all(|k| k.at.distance(s.at) > MARK_APART) {
            kept.push(s);
        }
    }
    kept
}
