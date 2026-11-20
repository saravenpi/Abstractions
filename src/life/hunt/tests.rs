//! Tests of hunts: determinism, prey lists, kills that take a member and give it back, smooth chases.

use bevy::math::DVec3;

use super::{Hunt, Outcome, RECOVERY, carcasses_near, hunts_near};
use crate::life::Life;
use crate::life::herd::fixtures::{light, meadow_life, sample};
use crate::life::herd::{Ground, Herd, Search, arc, member, near};

/// Radius of the test planet in metres.
pub(super) const RADIUS: f64 = 6000.0;

/// Returns a point on land of the test planet.
pub(super) fn land() -> DVec3 {
    DVec3::new(1.0, 0.3, 0.2).normalize()
}

/// Returns the search of `radius` metres around `at` at `time`.
pub(super) fn search(at: DVec3, radius: f64, time: f64) -> Search {
    Search {
        at,
        radius,
        planet: RADIUS,
        time,
    }
}

/// Returns the hunts seen around [`land`] over two hours, each once.
pub(super) fn hunts(life: &Life) -> Vec<Hunt> {
    let ground = Ground {
        sample: &|d| sample(d, 1.0),
        light: &light,
    };
    let mut all: Vec<Hunt> = Vec::new();
    for step in 0..48 {
        for hunt in hunts_near(
            life,
            &ground,
            &search(land(), 2500.0, 150.0 * f64::from(step)),
        ) {
            if !all
                .iter()
                .any(|h| h.hunter == hunt.hunter && h.start == hunt.start)
            {
                all.push(hunt);
            }
        }
    }
    all
}

/// Returns herd `id` at `time`, looked for within 2 km of `at`.
pub(super) fn herd(life: &Life, id: u64, at: DVec3, time: f64) -> Option<Herd> {
    let ground = Ground {
        sample: &|d| sample(d, 1.0),
        light: &light,
    };
    near(life, &ground, &search(at, 2000.0, time))
        .into_iter()
        .find(|h| h.id == id)
}

/// The same life and time give the same hunts and carcasses, and hunters only go for their prey.
#[test]
fn same_seed_same_hunts_on_prey_only() {
    let (a, b) = (hunts(&meadow_life()), hunts(&meadow_life()));
    assert!(a.len() >= 3, "{} hunts", a.len());
    assert_eq!(a, b);
    let life = meadow_life();
    for hunt in &a {
        let hunter = &life.species[usize::from(hunt.hunter_species)];
        assert!(hunter.prey.contains(&hunt.prey_species), "{hunt:?}");
    }
    let ground = Ground {
        sample: &|d| sample(d, 1.0),
        light: &light,
    };
    let at = search(land(), 2500.0, a[0].outcome_time() + 60.0);
    assert_eq!(
        carcasses_near(&life, &ground, &at),
        carcasses_near(&meadow_life(), &ground, &at)
    );
}

/// Returns the first kill around [`land`] and the slot it takes.
fn first_kill(life: &Life) -> (Hunt, u8) {
    let kill = hunts(life).into_iter().find_map(|h| match h.outcome {
        Outcome::Kill { slot, .. } => Some((h, slot)),
        Outcome::Escape => None,
    });
    kill.expect("a kill")
}

/// A killed member is gone from its herd from the kill time, and back once the herd has recovered.
#[test]
fn kills_take_a_member_and_recover() {
    let life = meadow_life();
    let (hunt, slot) = first_kill(&life);
    let time = hunt.outcome_time();
    let before = herd(&life, hunt.prey, hunt.to, hunt.start).expect("prey before");
    let after = herd(&life, hunt.prey, hunt.to, time + 1.0).expect("prey after");
    assert!(member(&before, slot, time - 0.5).is_some());
    assert!(
        member(&before, slot, time + 0.5).is_none(),
        "the herd knows the kill to come"
    );
    assert!(member(&after, slot, time + 1.0).is_none());
    assert_eq!(after.count + 1, before.count);
    let later = herd(&life, hunt.prey, hunt.to, time + RECOVERY + 1.0).expect("prey recovered");
    assert!(member(&later, slot, time + RECOVERY + 1.0).is_some());
}

/// A kill leaves a carcass at its place, that the scavenger comes to, and bones after the meat.
#[test]
fn kills_leave_carcasses() {
    let life = meadow_life();
    let (hunt, slot) = first_kill(&life);
    let ground = Ground {
        sample: &|d| sample(d, 1.0),
        light: &light,
    };
    let around = |time| carcasses_near(&life, &ground, &search(hunt.to, 50.0, time));
    let fresh = around(hunt.outcome_time() + 1.0);
    let carcass = fresh
        .iter()
        .find(|c| c.herd == hunt.prey && c.slot == slot)
        .expect("a carcass");
    assert!(carcass.visitors.contains(&8) && carcass.meat < carcass.bones);
    assert!(arc(carcass.at, hunt.to) < 1e-12);
    assert!(
        around(carcass.bones + 1.0)
            .iter()
            .all(|c| c.since != carcass.since)
    );
}
