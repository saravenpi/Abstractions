//! Droppings and grazed patches: what a herd leaves where it stood over the last [`LINGER`] seconds.
//!
//! The herd is read every [`LINGER_STEP`] seconds on the universe clock. At each read it stands still,
//! each member may leave a dropping by a seeded roll; plant eaters that graze there leave one patch of
//! cropped plants, as fresh as the last read they grazed it.

use super::tune::{DUNG_CHANCE, DUNG_PER_KG, LINGER, LINGER_STEP};
use super::{Mark, Sight, Sign, beside, toward};
use crate::life::herd::{self, Herd, HerdState, Watch, arc};
use crate::life::{Guild, Life};
use crate::universe::hash::{Rng, child};

/// Salt of the droppings rolls.
const DUNG: u64 = 0xD0_6D06;

/// Returns true when `herd` stands still at a waypoint.
fn standing(herd: &Herd) -> bool {
    herd.speed < 0.05
        && matches!(
            herd.state,
            HerdState::Graze | HerdState::Drink | HerdState::Rest | HerdState::Sleep
        )
}

/// Adds the droppings and grazed patches herd `id` of `watch` left within the search radius over
/// the last [`LINGER`] seconds.
pub(super) fn lay(watch: &mut Watch, life: &Life, id: u64, sight: &Sight, out: &mut Vec<Sign>) {
    let now = sight.search.time;
    let first = ((now - LINGER) / LINGER_STEP).ceil() as i64;
    let mut patch = None;
    for read in first..=(now / LINGER_STEP).floor() as i64 {
        let time = read as f64 * LINGER_STEP;
        let Some(herd) = watch.herd(life, sight.ground, id, time).filter(standing) else {
            continue;
        };
        if !sight.within(herd.centre, f64::from(herd.spacing) * 8.0) {
            continue;
        }
        let fresh = (1.0 - (now - time) / LINGER) as f32;
        dung(&herd, (read, time, fresh), sight, out);
        if herd.state == HerdState::Graze && crops(sight.species.role.guild) {
            patch = graze(&herd, (patch, fresh), sight, out);
        }
    }
}

/// Returns true when a species of `guild` crops plants where it feeds.
fn crops(guild: Guild) -> bool {
    matches!(guild, Guild::Grazer | Guild::Browser | Guild::Granivore)
}

/// Adds the droppings `herd` leaves at read number `read`, universe `time`, as fresh as `fresh`.
fn dung(herd: &Herd, (read, time, fresh): (i64, f64, f32), sight: &Sight, out: &mut Vec<Sign>) {
    let size = DUNG_PER_KG * sight.species.history.mass.cbrt();
    for slot in 0..herd.base {
        let mut rng = Rng::new(child(herd.id ^ DUNG, (read as u64) << 8 | u64::from(slot)));
        if rng.unit() >= DUNG_CHANCE {
            continue;
        }
        let Some(member) = herd::member(herd, slot, time) else {
            continue;
        };
        let at = beside(herd.centre, member.offset, sight.search.planet);
        let sign = Sign {
            mark: Mark::Dropping,
            species: herd.species,
            at,
            facing: toward(at, rng.unit() * std::f64::consts::TAU),
            size,
            fresh,
        };
        sight.keep(sign, out);
    }
}

/// Adds or refreshes the grazed patch of `herd`, and returns its index in `out`: the patch at index
/// `patch` when the herd still grazes there, else a new one, as fresh as `fresh`.
fn graze(
    herd: &Herd,
    (patch, fresh): (Option<usize>, f32),
    sight: &Sight,
    out: &mut Vec<Sign>,
) -> Option<usize> {
    let size = 2.0 * herd.spacing * f32::from(herd.count.max(1)).sqrt();
    let near = |s: &Sign| arc(s.at, herd.centre) * sight.search.planet < f64::from(herd.spacing);
    if let Some(sign) = patch.and_then(|i| out.get_mut(i)).filter(|s| near(s)) {
        sign.fresh = fresh;
        return patch;
    }
    let sign = Sign {
        mark: Mark::Grazed,
        species: herd.species,
        at: herd.centre,
        facing: herd.heading,
        size,
        fresh,
    };
    let len = out.len();
    sight.keep(sign, out);
    (out.len() > len).then_some(len)
}
