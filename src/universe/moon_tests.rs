//! Moon tests: moons repeat, form three ways, fit between the Roche limit and the Hill sphere
//! and face their planet.

use std::f64::consts::TAU;
use std::time::Instant;

use bevy::math::DQuat;

use super::giant_tests::{giant, systems};
use super::hill::{
    CAPTURE_SHARE, HILL_SHARE, IMPACT_ORBIT, IMPACT_SHARE, MOST, REGULAR_MOONS, SPACING, hill,
};
use super::*;

/// Returns every planet of the sample that has moons, with its star and its moons.
pub(super) fn hosts() -> Vec<(StarSpec, PlanetSpec, Vec<PlanetSpec>)> {
    let mut hosts = Vec::new();
    for system in systems() {
        for p in system.planets.into_iter().filter(|p| p.moons > 0) {
            let moons = moon::of(&system.star, &p);
            hosts.push((system.star.clone(), p, moons));
        }
    }
    hosts
}

/// The same seed gives the same moons, and looking a moon up by its id finds the same spec.
#[test]
fn same_seed_same_moons() {
    for (star, p, moons) in hosts().into_iter().take(200) {
        assert_eq!(moons, moon::of(&star, &p));
        assert_eq!(usize::from(p.moons), moons.len());
        for (moon, rank) in moons.iter().zip(1u8..) {
            assert_eq!(moon::ranked(&star, &p, rank).as_ref(), Some(moon));
        }
        assert_eq!(moon::ranked(&star, &p, p.moons + 1), None);
        assert_eq!(moon::ranked(&star, &p, 0), None);
    }
}

/// Every moon orbits outside the Roche limit of its planet and inside half its Hill sphere,
/// and each one at least `SPACING` times further out than the one before.
#[test]
fn moon_orbits_fit_inside_the_hill_sphere() {
    for (star, p, moons) in hosts() {
        let reach = HILL_SHARE * hill(&star, &p);
        for m in &moons {
            assert!(m.orbit_radius > ROCHE * p.radius, "{m:?}");
            assert!(m.orbit_radius < reach, "{m:?}");
        }
        let spaced = |pair: &[PlanetSpec]| pair[1].orbit_radius > SPACING * pair[0].orbit_radius;
        assert!(moons.windows(2).all(spaced), "{p:?}");
    }
}

/// Every moon turns once per orbit, the same way round, takes the year its planet's mass
/// gives it, and has no moons or rings of its own.
#[test]
fn moons_are_tidally_locked() {
    for (star, p, moons) in hosts() {
        for (m, rank) in moons.iter().zip(1u8..) {
            assert_eq!(m.day_seconds, m.orbit_seconds);
            assert!(m.locked());
            assert_eq!(m.orbit_seconds, orbit_seconds(p.mu(), m.orbit_radius));
            assert_eq!(m.host, Some(p.orbit()));
            assert_eq!(m.id, BodyId { moon: rank, ..p.id });
            assert_eq!((m.moons, m.rings), (0, None));
            assert!(m.name.starts_with(&p.name) && m.name.len() > p.name.len());
            assert!(moon::of(&star, m).is_empty());
        }
    }
}

/// Seen from a moon's own turning ground, the direction to its planet stays fixed over a whole
/// orbit: the ground turns as the world turns a body, `day_seconds` per turn about +Y.
#[test]
fn moons_keep_one_face_to_their_planet() {
    for (_, _, moons) in hosts().into_iter().take(300) {
        for m in &moons {
            let seen = |time: f64| {
                let spin = DQuat::from_rotation_y(TAU * time / m.day_seconds);
                spin.inverse() * -m.orbit().at(time).normalize()
            };
            let first = seen(0.0);
            for step in 1..=8 {
                let time = m.orbit_seconds * f64::from(step) / 8.0;
                assert!(seen(time).distance(first) < 1e-9, "{m:?}");
            }
        }
    }
}

/// Moons form three ways. Among the solid planets with room for any moon, about a quarter
/// hold one large impact moon and about 15 in 100 hold one or two small captured ones, within
/// 5 points. A gas giant with room holds 2 to 6 moons, of which 2 or more are regular.
#[test]
fn moons_form_three_ways() {
    let (mut roomy, mut impact, mut caught) = (0.0, 0.0, 0.0);
    for system in systems() {
        for p in &system.planets {
            let moons = moon::of(&system.star, p);
            let room = HILL_SHARE * hill(&system.star, p) > 2.0 * IMPACT_ORBIT.1 * p.radius;
            let large = |m: &PlanetSpec| m.radius >= IMPACT_RADIUS.0 * p.radius;
            let small = |m: &PlanetSpec| m.radius <= CAPTURED_RADIUS.1;
            if giant(p) && room {
                let regular = moons.iter().filter(|m| m.radius >= REGULAR_RADIUS.0);
                assert!(regular.count() >= usize::from(REGULAR_MOONS.0), "{p:?}");
                assert!(moons.len() <= MOST, "{p:?}");
            } else if room {
                assert!(moons.len() <= 2 && (moons.iter().all(large) || moons.iter().all(small)));
                roomy += 1.0;
                impact += f64::from(u8::from(moons.len() == 1 && large(&moons[0])));
                caught += f64::from(u8::from(!moons.is_empty() && small(&moons[0])));
            }
        }
    }
    let (impact, caught) = (impact / roomy, caught / roomy);
    println!("of {roomy} roomy solid planets: impact moon {impact:.3}, captured {caught:.3}");
    assert!((impact - IMPACT_SHARE).abs() < 0.05, "{impact}");
    assert!((caught - CAPTURE_SHARE).abs() < 0.05, "{caught}");
}

/// Looking up a moon builds one system and the moons of one planet: tens of microseconds.
#[test]
fn moon_lookup_is_cheap() {
    let (_, planet, _) = hosts().swap_remove(0);
    let u = Universe::new(0);
    let id = BodyId {
        moon: 1,
        ..planet.id
    };
    let run = || {
        let start = Instant::now();
        let found = (0..200).map(|_| u.planet(id).radius).sum::<f64>();
        (start.elapsed() / 200, found)
    };
    let (best, found) = (0..5)
        .map(|_| run())
        .min_by_key(|r| r.0)
        .expect("five runs");
    println!("one moon lookup: {best:?}");
    assert!(found > 0.0);
    assert!(best.as_micros() < 120, "a moon lookup took {best:?}");
}
