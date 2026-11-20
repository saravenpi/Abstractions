//! Bones: what a kill leaves once its meat is gone, until the bones are gone too.

use super::{Mark, Sign, toward};
use crate::life::Life;
use crate::life::herd::{Search, arc};
use crate::life::hunt::Carcass;
use crate::universe::hash::{Rng, child};

/// Salt of the lie of the bones.
const LIE: u64 = 0xB0_4E5;

/// Adds the bones of `carcasses` whose meat is gone at the search time and that lie within the
/// search radius: as long as the body was, fading as they weather away.
pub(super) fn lay(life: &Life, carcasses: &[Carcass], search: &Search, out: &mut Vec<Sign>) {
    let time = search.time;
    for c in carcasses {
        let bare = c.meat <= time && time < c.bones;
        if !bare || arc(c.at, search.at) * search.planet > search.radius {
            continue;
        }
        let Some(species) = life.species.get(usize::from(c.species)) else {
            continue;
        };
        let angle = Rng::new(child(c.herd ^ LIE, u64::from(c.slot))).unit() * std::f64::consts::TAU;
        let age = (time - c.meat) / (c.bones - c.meat).max(1.0);
        out.push(Sign {
            mark: Mark::Bones,
            species: c.species,
            at: c.at,
            facing: toward(c.at, angle),
            size: species.body.length.max(0.2),
            fresh: (1.0 - age).sqrt() as f32,
        });
    }
}
