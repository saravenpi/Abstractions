//! Nests and burrows: where species that tend their young raise them, at the home of each herd.
//!
//! Small ground animals dig a warren of burrows; larger ones and shore animals scrape a nest, and
//! flyers build one on the ground under their range. Animals of the water raise their young in it,
//! out of sight. Each den lies at a seeded place near the home of its herd, on dry ground only.

use super::tune::{BURROW_KG, BURROWS, NEST_CARE};
use super::{Mark, Sight, Sign, beside, toward};
use crate::life::herd::Herd;
use crate::life::{Habitat, Species};
use crate::universe::hash::{Rng, child};

/// Salt of the den rolls.
const DEN: u64 = 0xDE_4D0E;

/// Returns the kind of den of `species` and how many a herd of `base` members keeps, or nothing
/// when it keeps none.
fn den(species: &Species, base: u8) -> Option<(Mark, u8)> {
    if species.history.care < NEST_CARE {
        return None;
    }
    match species.role.habitat {
        Habitat::Water => None,
        Habitat::Ground if species.history.mass <= BURROW_KG => {
            Some((Mark::Burrow, (base / 3).clamp(1, BURROWS)))
        }
        _ => Some((Mark::Nest, 1 + u8::from(base > 8))),
    }
}

/// Returns the length in metres of a den of kind `mark` for an adult of `species`: a nest a little
/// longer than the body, a burrow mouth as wide as the body is tall.
fn size(mark: Mark, species: &Species) -> f32 {
    let body = &species.body;
    match mark {
        Mark::Burrow => (0.8 * body.height).max(0.06),
        _ => (1.3 * body.length).max(0.15),
    }
}

/// Adds the dens of `herd` that lie within the search radius on dry ground.
pub(super) fn lay(herd: &Herd, sight: &Sight, out: &mut Vec<Sign>) {
    let species = sight.species;
    let Some((mark, count)) = den(species, herd.base) else {
        return;
    };
    if !sight.within(herd.home, 30.0 * f64::from(herd.spacing)) {
        return;
    }
    let size = size(mark, species);
    for index in 0..count {
        let mut rng = Rng::new(child(herd.id ^ DEN, u64::from(index)));
        let reach = 3.0 * herd.spacing * (1.0 + f32::from(index)) * rng.unit().sqrt() as f32;
        let angle = rng.unit() * std::f64::consts::TAU;
        let offset = bevy::math::Vec2::from_angle(angle as f32) * reach;
        let at = beside(herd.home, offset, sight.search.planet);
        if !sight.within(at, 0.0) || (sight.ground.sample)(at).liquid {
            continue;
        }
        let facing = toward(at, rng.unit() * std::f64::consts::TAU);
        let sign = Sign {
            mark,
            species: herd.species,
            at,
            facing,
            size,
            fresh: 1.0,
        };
        out.push(sign);
    }
}
