//! Footprints: where the members of a herd stepped over the last [`TRAIL`] seconds.
//!
//! A print falls at every multiple of the step time of its species on the universe clock, so prints
//! stay where they are as time goes on and every player sees the same ones. The step time is one
//! step length over the walking speed, so a herd that runs leaves prints farther apart. The herd is
//! read every [`SAMPLE`] seconds and its centre is interpolated between two reads.

use bevy::math::DVec3;

use super::tune::{FOOT_PER_KG, MOVING, SAMPLE, STEP_PER_LEG, TRACKERS, TRAIL};
use super::{Mark, Sight, Sign, beside};
use crate::life::body::Locomotion;
use crate::life::herd::{self, Herd, HerdState, Watch, slerp};
use crate::life::{Habitat, Life, Species};

/// Returns the seconds between two prints of `species`, or nothing when it leaves none: it does not
/// walk on legs on the ground.
pub(super) fn step_time(species: &Species) -> Option<f64> {
    let ground = matches!(species.role.habitat, Habitat::Ground | Habitat::Shore);
    if !ground || species.body.locomotion != Locomotion::Walk {
        return None;
    }
    let walk = (f64::from(species.history.speed) * 0.125).max(0.3);
    Some(f64::from(STEP_PER_LEG * leg(species)) / walk)
}

/// Returns the length in metres of a leg of `species`: its first leg, or its body height.
fn leg(species: &Species) -> f32 {
    species
        .body
        .legs
        .first()
        .map_or(species.body.height, |l| l.lengths.iter().sum())
        .max(0.05)
}

/// Returns true when `herd` walks at `time` fast enough to leave prints.
fn walking(herd: &Herd) -> bool {
    matches!(
        herd.state,
        HerdState::Travel | HerdState::Chase | HerdState::Stalk
    ) && herd.speed >= MOVING
}

/// Adds the prints herd `id` of `watch` left within the search radius over the last [`TRAIL`]
/// seconds.
pub(super) fn lay(watch: &mut Watch, life: &Life, id: u64, sight: &Sight, out: &mut Vec<Sign>) {
    let Some(step) = step_time(sight.species) else {
        return;
    };
    let now = sight.search.time;
    let first = ((now - TRAIL) / SAMPLE).floor() as i64;
    let mut before = watch.herd(life, sight.ground, id, first as f64 * SAMPLE);
    for read in first + 1..=(now / SAMPLE).ceil() as i64 {
        let after = watch.herd(life, sight.ground, id, read as f64 * SAMPLE);
        if let (Some(a), Some(b)) = (&before, &after) {
            stretch((a, b), (read as f64 - 1.0) * SAMPLE, step, sight, out);
        }
        before = after;
    }
}

/// Returns true when the herd walks at its read `a` and one of its reads `a` and `b` is near enough
/// the search for its members' prints to fall inside it.
fn seen((a, b): (&Herd, &Herd), sight: &Sight) -> bool {
    let slack = f64::from(a.spacing) * 6.0 + 10.0;
    walking(a) && (sight.within(a.centre, slack) || sight.within(b.centre, slack))
}

/// Adds the prints of the herd between its reads `a` at universe time `from` and `b` [`SAMPLE`]
/// seconds later, one every `step` seconds.
fn stretch((a, b): (&Herd, &Herd), from: f64, step: f64, sight: &Sight, out: &mut Vec<Sign>) {
    let (planet, now) = (sight.search.planet, sight.search.time);
    if !seen((a, b), sight) {
        return;
    }
    let size = FOOT_PER_KG * sight.species.history.mass.cbrt();
    let start = (from.max(now - TRAIL) / step).ceil() as i64;
    let end = (((from + SAMPLE) / step).ceil() as i64).min((now / step).floor() as i64 + 1);
    for k in start..end {
        let time = k as f64 * step;
        let centre = slerp(a.centre, b.centre, (time - from) / SAMPLE);
        let fresh = (1.0 - (now - time) / TRAIL) as f32;
        let print = Print {
            centre,
            k,
            fresh,
            size,
        };
        for slot in 0..a.base.min(TRACKERS) {
            if let Some(sign) = print.of(a, slot, (time, planet), sight.species) {
                sight.keep(sign, out);
            }
        }
    }
}

/// One step of a herd: where its centre is, the step number, and the freshness and size of the prints.
struct Print {
    /// Unit direction of the herd centre.
    centre: DVec3,
    /// Step number on the universe clock.
    k: i64,
    /// Freshness of the prints, 1 new to 0 gone.
    fresh: f32,
    /// Length of a print in metres.
    size: f32,
}

impl Print {
    /// Returns the print member `slot` of `herd` leaves at this step at universe `time` on a planet of
    /// `planet` metres, left and right feet in turn, or nothing when that member is dead.
    fn of(
        &self,
        herd: &Herd,
        slot: u8,
        (time, planet): (f64, f64),
        species: &Species,
    ) -> Option<Sign> {
        let member = herd::member(herd, slot, time)?;
        let at = beside(self.centre, member.offset, planet);
        let left = at.cross(herd.heading).normalize_or_zero();
        let foot = if (self.k + i64::from(slot)) % 2 == 0 {
            1.0
        } else {
            -1.0
        };
        let gauge = 0.15 * f64::from(species.body.length.max(0.1)) * foot;
        let size = if member.young {
            0.6 * self.size
        } else {
            self.size
        };
        Some(Sign {
            mark: Mark::Print,
            species: herd.species,
            at: (at + left * gauge / planet).normalize(),
            facing: herd.heading,
            size,
            fresh: self.fresh,
        })
    }
}
