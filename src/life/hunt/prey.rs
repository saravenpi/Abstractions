//! Prey: which herd a hunter goes for, which member it takes, and where that member falls.

use bevy::math::{DVec2, DVec3};

use super::{Hunt, Outcome, TERRITORY};
use crate::life::herd::{Roam, Scene, east, offset, spacing};
use crate::life::{Guild, Species};
use crate::universe::hash::Rng;
use crate::world::globe::Tangent;

/// Returns the prey herd of hunter `roam` of `species` nearest to `at` at `time`, among the herds of
/// its prey species living within [`TERRITORY`] of its home and within `reach` metres of `at`, and
/// where it is then.
pub(super) fn target<'s>(
    scene: &'s Scene,
    (roam, species): (&Roam, &Species),
    at: DVec3,
    time: f64,
    reach: f64,
) -> Option<(&'s Roam, DVec3)> {
    scene
        .roams
        .iter()
        .filter(|r| species.prey.contains(&r.species))
        .filter(|r| scene.metres(r.home, roam.home) <= TERRITORY)
        .filter(|r| scene.metres(r.home, at) <= reach + r.reach)
        .filter_map(|r| Some((r, scene.pose(r, time)?.centre)))
        .map(|(r, c)| (scene.metres(c, at), r, c))
        .filter(|(d, _, _)| *d <= reach)
        .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.id.cmp(&b.1.id)))
        .map(|(_, r, c)| (r, c))
}

/// Ends `hunt` of `hunter` on `prey`, begun from `distance` metres by a hunter hunting every `every`
/// seconds: on a kill, the chase ends where the killed member stands and the hunters feed.
pub(super) fn strike(
    scene: &Scene,
    hunt: Hunt,
    (hunter, prey): (&Species, &Roam),
    rng: &mut Rng,
    (distance, every): (f64, f64),
) -> Option<Hunt> {
    let kind = scene.species(prey)?;
    let time = hunt.outcome_time();
    let centre = scene.pose(prey, time)?.centre;
    let Some(slot) = kill(rng, (hunter, prey, kind), distance) else {
        return Some(Hunt { to: centre, ..hunt });
    };
    let o = offset(prey.id, spacing(kind), slot, time).as_dvec2();
    let at =
        Tangent::new(centre, east(centre), scene.planet, DVec2::ZERO).dir(DVec2::new(o.x, -o.y));
    Some(Hunt {
        feed: 0.15 * every,
        outcome: Outcome::Kill { slot, at },
        to: at,
        ..hunt
    })
}

/// Rolls the outcome of a hunt of `hunter` on `prey` of `kind` from `distance` metres: the slot killed,
/// or nothing on an escape. Faster hunters do better, the young and the herd edge are easier, and
/// ambushers strike best from close. The pinned herd never loses its first three slots.
fn kill(
    rng: &mut Rng,
    (hunter, prey, kind): (&Species, &Roam, &Species),
    distance: f64,
) -> Option<u8> {
    let lowest = if prey.pinned { 3 } else { 0 };
    let count = prey.count;
    let edge = count - count.div_ceil(3);
    let (low, high, ease) = match rng.unit() {
        r if r < 0.4 && prey.young > lowest => (lowest, prey.young, 1.6),
        r if r < 0.8 => (edge.max(lowest), count, 1.3),
        _ => (lowest, count, 1.0),
    };
    if low >= high {
        return None;
    }
    let slot = low + rng.below(u32::from(high - low)) as u8;
    let ratio = f64::from(hunter.history.speed) / f64::from(kind.history.speed.max(0.1));
    let close = if hunter.role.guild == Guild::Ambusher && distance < 30.0 {
        1.5
    } else {
        1.0
    };
    let odds = (0.3 * ratio.powf(1.5) * ease * close).clamp(0.05, 0.85);
    (rng.unit() < odds).then_some(slot)
}
