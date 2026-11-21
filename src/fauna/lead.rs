//! Where an animal should be at a universe time: its place in its herd, bent towards the kill when it
//! is the one hunted down, or its place around the carcass it visits.

use bevy::math::{DVec3, Vec2};

use super::herds::member;
use super::place::{beside, east};
use super::tune::{GRAZE_EVERY, GRAZE_REACH, RING, SOAR_LOOP, SOAR_SPEED};
use super::want::Lead;
use crate::life::Species;
use crate::life::body::Locomotion;
use crate::life::herd::{Herd, HerdState};
use crate::life::hunt::{Hunt, Outcome};
use crate::universe::hash::{Rng, child};

/// Salt of the beats of the grazing steps, so they draw apart from the other rolls of a herd.
const GRAZE_KEY: u64 = 0x6752_415A;

/// Where an animal should be, and how its group moves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Goal {
    /// Unit direction of its place.
    pub dir: DVec3,
    /// Velocity of its group in metres per second, a tangent in planet axes.
    pub flow: DVec3,
    /// Alarm of its group, 0 to 1: hunters on a chase, prey hunted down.
    pub excite: f32,
}

/// Returns how much a herd doing `state` is roused, 0 to 1.
fn roused(state: HerdState) -> f32 {
    match state {
        HerdState::Chase => 1.0,
        HerdState::Stalk => 0.3,
        _ => 0.0,
    }
}

/// Returns the share of the chase of `hunt` run at `time` if member `slot` of herd `id` is its kill.
fn hunted(hunt: &Hunt, (id, slot): (u64, u8), time: f64) -> Option<f64> {
    let kill = matches!(hunt.outcome, Outcome::Kill { slot: s, .. } if s == slot);
    let from = hunt.start + hunt.stalk;
    let on = hunt.prey == id && kill && (from..hunt.outcome_time()).contains(&time);
    on.then(|| ((time - from) / hunt.chase.max(1e-3)).clamp(0.0, 1.0))
}

/// Returns the offset in metres, x east and y north, at which grazing member `slot` of herd `id`
/// crops at universe `time`, around its place in a herd `spacing` metres apart: a new patch every
/// [`GRAZE_EVERY`] seconds, each member on its own beat, within [`GRAZE_REACH`] spacings of its place,
/// so a grazing herd steps about as it feeds.
fn grazing((id, slot): (u64, u8), spacing: f32, time: f64) -> Vec2 {
    let key = child(id ^ GRAZE_KEY, u64::from(slot));
    let beat = time / GRAZE_EVERY + Rng::new(key).unit();
    let mut rng = Rng::new(child(key, beat.max(0.0).floor() as u64));
    let angle = rng.unit() * std::f64::consts::TAU;
    let metres = f64::from(spacing * GRAZE_REACH) * rng.unit().sqrt();
    Vec2::new((metres * angle.cos()) as f32, (metres * angle.sin()) as f32)
}

/// Returns the goal of member `slot` of `herd` at `time` on a planet of `radius` metres: its place in
/// the herd, the patch it crops while the herd grazes, or the way to the kill point while hunters run
/// it down.
pub fn in_herd(herd: &Herd, slot: u8, hunts: &[Hunt], time: f64, radius: f64) -> Option<Goal> {
    let (_, place) = member(herd, slot, time, radius)?;
    let place = match herd.state {
        HerdState::Graze => beside(place, grazing((herd.id, slot), herd.spacing, time), radius),
        _ => place,
    };
    let chase = hunts
        .iter()
        .find_map(|h| hunted(h, (herd.id, slot), time).map(|u| (h.to, u)));
    let dir = chase.map_or(place, |(to, u)| {
        place.lerp(to, u * u * (3.0 - 2.0 * u)).normalize()
    });
    Some(Goal {
        dir,
        flow: herd.heading * f64::from(herd.speed),
        excite: if chase.is_some() {
            1.0
        } else {
            roused(herd.state)
        },
    })
}

/// Returns the goal of visitor `slot` of the carcass at unit direction `at` of identity `key`, of
/// `ring` metres around it, at `time` on a planet of `radius` metres: a seeded place on the ring that
/// slowly goes round.
pub fn visiting((key, slot, at): (u64, u8, DVec3), ring: f32, time: f64, radius: f64) -> Goal {
    let mut rng = Rng::new(child(key, u64::from(slot)));
    let angle = rng.unit() * std::f64::consts::TAU + 0.05 * time;
    let offset = Vec2::new(angle.cos() as f32, angle.sin() as f32) * ring;
    Goal {
        dir: beside(at, offset, radius),
        flow: DVec3::ZERO,
        excite: 0.0,
    }
}

/// Returns `goal` moved round a loop above its place, as flyer `slot` of flock `id` of `species`
/// circles at `time` on a planet of `radius` metres: the flock goes round together, each bird a
/// little behind the one before.
pub fn soar(goal: Goal, species: &Species, (id, slot): (u64, u8), time: f64, radius: f64) -> Goal {
    let size = f64::from(species.body.length);
    let round = f64::from(SOAR_LOOP) * (1.0 + size).min(4.0);
    let speed = f64::from((species.history.speed * SOAR_SPEED).clamp(2.0, 15.0));
    let start = Rng::new(child(id, 0x50A2)).unit() * std::f64::consts::TAU;
    let angle = start + speed / round * time - 0.25 * f64::from(slot);
    let (sin, cos) = angle.sin_cos();
    let (east, offset) = (
        east(goal.dir),
        Vec2::new(cos as f32, sin as f32) * round as f32,
    );
    let north = goal.dir.cross(east);
    Goal {
        dir: beside(goal.dir, offset, radius),
        flow: goal.flow + (north * cos - east * sin) * speed,
        ..goal
    }
}

/// Returns the goal of the animal led by `lead` of `species`, with its herd `herd` when it has one:
/// flyers circle above their flock's place.
pub fn goal(
    lead: &Lead,
    herd: Option<&Herd>,
    (hunts, species): (&[Hunt], &Species),
    time: f64,
    radius: f64,
) -> Option<Goal> {
    let ring = RING + species.body.length;
    let flies = species.body.locomotion == Locomotion::Fly;
    match *lead {
        Lead::Herd { id, slot } => in_herd(herd?, slot, hunts, time, radius)
            .map(|g| [g, soar(g, species, (id, slot), time, radius)][usize::from(flies)]),
        Lead::Visit { key, slot, at } => Some(visiting((key, slot, at), ring, time, radius)),
        Lead::Carcass { .. } => None,
    }
}
