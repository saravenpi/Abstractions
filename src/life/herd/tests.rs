//! Tests of herds: determinism, continuity, habitat, watches and the herds of real planets.

use bevy::math::DVec3;

use super::fixtures::{height, light, meadow_life, sample};
use super::sphere::arc;
use super::{Ground, Herd, HerdState, Search, Watch, member, near};
use crate::life::{Habitat, Life};

/// Radius of the test planet in metres.
pub(super) const RADIUS: f64 = 6000.0;

/// Returns a point on land of the test planet.
pub(super) fn land() -> DVec3 {
    DVec3::new(1.0, 0.3, 0.2).normalize()
}

/// Returns the herds of `life` within `radius` metres of `at` at `time` on the test planet.
fn look(life: &Life, at: DVec3, radius: f64, time: f64) -> Vec<Herd> {
    let search = Search {
        at,
        radius,
        planet: RADIUS,
        time,
    };
    let ground = Ground {
        sample: &|dir| sample(dir, 1.0),
        light: &light,
    };
    near(life, &ground, &search)
}

/// The same life and time give the same herds and members, built from scratch twice.
#[test]
fn same_seed_same_herds() {
    let a = look(&meadow_life(), land(), 1500.0, 12_345.6);
    let b = look(&meadow_life(), land(), 1500.0, 12_345.6);
    assert!(a.len() > 3, "{} herds", a.len());
    assert_eq!(a, b);
    let herd = &a[0];
    assert_eq!(member(herd, 2, 50.0), member(&b[0], 2, 50.0));
}

/// Herd centres never jump: between t and t + 0.1 s each moves at most its speed times 0.1 s, 5 percent
/// for the change of speed within the step, plus 1 cm.
#[test]
fn herds_move_continuously() {
    let life = meadow_life();
    let mut moving = 0;
    for step in 0..120 {
        let t = 37.0 * f64::from(step) * f64::from(step);
        let (now, next) = (
            look(&life, land(), 1200.0, t),
            look(&life, land(), 1200.0, t + 0.1),
        );
        for a in &now {
            let Some(b) = next.iter().find(|b| b.id == a.id) else {
                continue;
            };
            let jump = arc(a.centre, b.centre) * RADIUS;
            let bound = f64::from(a.speed.max(b.speed)) * 0.1 * 1.05 + 0.01;
            assert!(
                jump <= bound,
                "herd {} jumped {jump} m at {t} s (bound {bound})",
                a.id
            );
            moving += usize::from(jump > 0.0);
        }
    }
    assert!(moving > 10, "{moving} moving herds");
}

/// A ground herd never stays where the site is liquid.
#[test]
fn ground_herds_stay_dry() {
    let life = meadow_life();
    let mut seen = 0;
    for step in 0..60 {
        let t = 911.0 * f64::from(step);
        for herd in look(&life, land(), 2500.0, t) {
            let habitat = life.species[usize::from(herd.species)].role.habitat;
            if habitat == Habitat::Ground && herd.state != HerdState::Travel {
                seen += 1;
                assert!(
                    height(herd.centre) >= 0.0,
                    "herd {} in the sea at {t} s",
                    herd.id
                );
            }
        }
    }
    assert!(seen > 50, "{seen} resting ground herds");
}

/// Over the whole planet the apex hunter has the fewest herds of the ground animals.
#[test]
fn apex_herds_are_rarest() {
    let life = meadow_life();
    let herds = look(&life, land(), RADIUS * std::f64::consts::PI, 0.0);
    let count = |species| herds.iter().filter(|h| h.species == species).count();
    let apex = count(5);
    assert!(apex > 0);
    for species in 1..5 {
        assert!(
            count(species) > apex,
            "species {species}: {} vs apex {apex}",
            count(species)
        );
    }
}

/// The young stand nearer the centre than the adults, and members drift slowly.
#[test]
fn members_keep_the_young_inside() {
    let herds = look(&meadow_life(), land(), 2000.0, 500.0);
    let herd = herds
        .iter()
        .find(|h| h.young > 0 && h.count == h.base && h.count > h.young + 2)
        .copied();
    let Some(herd) = herd else {
        panic!("no herd with young")
    };
    let at = |slot, time| member(&herd, slot, time).expect("living member");
    let mean = |slots: &mut dyn Iterator<Item = u8>| {
        let all: Vec<f32> = slots.map(|s| at(s, 500.0).offset.length()).collect();
        all.iter().sum::<f32>() / all.len() as f32
    };
    assert!(mean(&mut (0..herd.young)) < mean(&mut (herd.young..herd.count)));
    assert!(at(0, 500.0).young && !at(herd.base - 1, 500.0).young);
    let drift = at(1, 500.0).offset.distance(at(1, 500.1).offset);
    assert!(drift < 0.05 * herd.spacing, "{drift}");
}

/// A watch rolled once answers at a later time exactly as a search made from scratch then.
#[test]
fn watch_matches_a_fresh_search() {
    let life = meadow_life();
    let ground = Ground {
        sample: &|dir| sample(dir, 1.0),
        light: &light,
    };
    let search = |time| Search {
        at: land(),
        radius: 600.0,
        planet: RADIUS,
        time,
    };
    let mut watch = Watch::new(&life, &ground, &search(3000.0));
    for step in 0..12 {
        let time = 3000.0 + 10.0 * f64::from(step);
        assert!(watch.covers(time));
        let herds = watch.herds(&life, &ground, time);
        assert_eq!(herds, near(&life, &ground, &search(time)), "at {time} s");
        let first = herds.first().expect("a herd");
        assert_eq!(watch.herd(&life, &ground, first.id, time), Some(*first));
    }
}
