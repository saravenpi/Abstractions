//! Tests of herds: determinism, continuity, habitat and the pinned herd.

use bevy::math::DVec3;

use super::fixtures::{height, meadow_life, sample};
use super::sphere::arc;
use super::{Herd, HerdState, Search, member, near};
use crate::life::{Habitat, Life};

/// Radius of the test planet in metres.
const RADIUS: f64 = 6000.0;

/// Returns a point on land of the test planet.
fn land() -> DVec3 {
    DVec3::new(1.0, 0.3, 0.2).normalize()
}

/// Returns the herds of `life` within `radius` metres of `at` at `time`, under noon light.
fn look(life: &Life, at: DVec3, radius: f64, time: f64, pin: Option<DVec3>) -> Vec<Herd> {
    let search = Search {
        at,
        radius,
        planet: RADIUS,
        time,
        pin,
    };
    near(life, &|dir| sample(dir, 1.0), &search)
}

/// The same life and time give the same herds and members, built from scratch twice.
#[test]
fn same_seed_same_herds() {
    let a = look(&meadow_life(), land(), 1500.0, 12_345.6, Some(land()));
    let b = look(&meadow_life(), land(), 1500.0, 12_345.6, Some(land()));
    assert!(a.len() > 3, "{} herds", a.len());
    assert_eq!(a, b);
    let herd = &a[0];
    assert_eq!(member(herd, 2, 50.0), member(&b[0], 2, 50.0));
}

/// Herd centres never jump: between t and t + 0.1 s each moves at most its speed times 0.1 s plus 1 cm.
#[test]
fn herds_move_continuously() {
    let life = meadow_life();
    let mut moving = 0;
    for step in 0..120 {
        let t = 37.0 * f64::from(step) * f64::from(step);
        let (now, next) = (
            look(&life, land(), 1200.0, t, None),
            look(&life, land(), 1200.0, t + 0.1, None),
        );
        for a in &now {
            let Some(b) = next.iter().find(|b| b.id == a.id) else {
                continue;
            };
            let jump = arc(a.centre, b.centre) * RADIUS;
            let bound = f64::from(a.speed.max(b.speed)) * 0.1 + 0.01;
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
        for herd in look(&life, land(), 2500.0, t, None) {
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
    let herds = look(&life, land(), RADIUS * std::f64::consts::PI, 0.0, None);
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

/// The pinned herd of the most common grazer is within 300 m of the pin at every time.
#[test]
fn pinned_herd_stays_near_the_pin() {
    let life = meadow_life();
    let pin = land();
    let mut ids = Vec::new();
    for step in 0..20 {
        let t = 1234.5 * f64::from(step) + 17.0;
        let herds = look(&life, pin, 400.0, t, Some(pin));
        let near: Vec<_> = herds
            .iter()
            .filter(|h| h.species == 1 && arc(h.home, pin) < 1e-9)
            .collect();
        assert_eq!(near.len(), 1, "at {t} s");
        assert!(arc(near[0].centre, pin) * RADIUS <= 300.0, "at {t} s");
        ids.push(near[0].id);
    }
    assert!(ids.windows(2).all(|w| w[0] == w[1]));
}

/// The young stand nearer the centre than the adults, and members drift slowly.
#[test]
fn members_keep_the_young_inside() {
    let herds = look(&meadow_life(), land(), 2000.0, 500.0, Some(land()));
    let herd = herds
        .iter()
        .find(|h| h.young > 0 && h.count > h.young + 2)
        .copied();
    let Some(herd) = herd else {
        panic!("no herd with young")
    };
    let mean = |slots: &mut dyn Iterator<Item = u8>| {
        let all: Vec<f32> = slots
            .map(|s| member(&herd, s, 500.0).offset.length())
            .collect();
        all.iter().sum::<f32>() / all.len() as f32
    };
    assert!(mean(&mut (0..herd.young)) < mean(&mut (herd.young..herd.count)));
    assert!(member(&herd, 0, 500.0).young && !member(&herd, herd.count - 1, 500.0).young);
    let drift = member(&herd, 1, 500.0)
        .offset
        .distance(member(&herd, 1, 500.1).offset);
    assert!(drift < 0.05 * herd.spacing, "{drift}");
}
