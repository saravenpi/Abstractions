//! Moon tests: moons repeat, fit inside the Hill sphere without crossing and face their planet.

use std::time::Instant;

use super::giant_tests::{giant, systems};
use super::hill::{HILL_SHARE, INNERMOST, SPACING, hill};
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
    for seed in 0..20 {
        let (a, b) = (Universe::new(seed), Universe::new(seed));
        let home = a.home();
        for index in 0..a.system(home.star).planets.len() as u8 {
            let id = BodyId { index, ..home };
            let moons = a.moons(id);
            assert_eq!(moons, b.moons(id));
            assert_eq!(a.planet(id).moons as usize, moons.len());
            for moon in &moons {
                assert_eq!(&a.planet(moon.id), moon);
                assert_eq!(a.moons(moon.id), moons);
            }
            let past = BodyId { moon: 7, ..id };
            assert_eq!(a.planet(past), a.planet(id));
        }
    }
}

/// Moons orbit outward from 4 planet radii to a third of the Hill radius, and each one clears
/// the one inside it by 4 times the sum of their radii, so no two orbits cross.
#[test]
fn moon_orbits_fit_inside_the_hill_sphere() {
    for (star, p, moons) in hosts() {
        let high = HILL_SHARE * hill(&star, &p);
        assert_eq!(usize::from(p.moons), moons.len());
        assert!(moons[0].orbit_radius >= INNERMOST * p.radius, "{p:?}");
        assert!(
            moons.iter().all(|m| m.orbit_radius + m.radius <= high),
            "{p:?}"
        );
        for pair in moons.windows(2) {
            let gap = pair[1].orbit_radius - pair[0].orbit_radius;
            let least = SPACING * (pair[0].radius + pair[1].radius);
            assert!(gap >= least * (1.0 - 1e-9), "{pair:?}");
        }
        let mass = p.mu() / GRAVITATION;
        let reach = p.orbit_radius * (mass / (3.0 * star.mass)).cbrt();
        assert!((reach / hill(&star, &p) - 1.0).abs() < 1e-9);
    }
}

/// A moon turns once per orbit around its planet, on an orbit set by the pull of its planet,
/// and it carries the orbit of its planet as its host. It has no moons and no rings.
#[test]
fn moons_are_tidally_locked() {
    for (star, p, moons) in hosts() {
        for (rank, m) in (1..).zip(&moons) {
            assert_eq!(m.day_seconds, m.orbit_seconds);
            assert_eq!(m.orbit_seconds, orbit_seconds(p.mu(), m.orbit_radius));
            assert_eq!(m.host, Some(p.orbit()));
            assert_eq!(m.id, BodyId { moon: rank, ..p.id });
            assert_eq!((m.moons, m.rings), (0, None));
            assert!(m.name.starts_with(&p.name) && m.name.len() > p.name.len());
            assert!(moon::of(&star, m).is_empty());
        }
    }
}

/// Gas giants have 2 to 6 moons and solid planets 0 to 2, most of them none and the bigger ones
/// more often. Counts are printed.
#[test]
fn moon_counts_follow_the_planet() {
    let (mut giants, mut solid) = ([0; 7], [0; 3]);
    let (mut small, mut big) = ((0, 0), (0, 0));
    for system in systems() {
        for p in &system.planets {
            match giant(p) {
                true => giants[usize::from(p.moons)] += 1,
                false => solid[usize::from(p.moons)] += 1,
            }
            let side = if p.radius < 10_000.0 {
                &mut small
            } else {
                &mut big
            };
            *side = (
                side.0 + usize::from(p.moons > 0 && !giant(p)),
                side.1 + usize::from(!giant(p)),
            );
        }
    }
    println!("giants by moon count {giants:?}, solid planets {solid:?}");
    assert_eq!(giants[..2], [0, 0]);
    assert!(solid[0] > solid[1] + solid[2] && solid[2] > 0, "{solid:?}");
    let rate = |(with, all): (usize, usize)| with as f64 / all as f64;
    assert!(rate(big) > 2.0 * rate(small), "{small:?} {big:?}");
}

/// Tides heat the closest moons of the heaviest giants: some are molten, some hold liquid water
/// under thin air, and the heat falls as the sixth power of the orbit radius. Classes of moons are printed.
#[test]
fn tides_warm_close_moons() {
    let mut classes = [0; 7];
    for (_, p, moons) in hosts() {
        for m in &moons {
            classes[m.record.class as usize] += 1;
            let heat = moon::tide(&p, m.orbit_radius);
            assert!(heat <= moon::TIDE, "{m:?}");
            let further = moon::tide(&p, 2.0 * m.orbit_radius);
            assert!((64.0 * further / heat - 1.0).abs() < 1e-9);
        }
    }
    println!("moons, ocean to gas giant: {classes:?}");
    let class = |c: PlanetClass| classes[c as usize];
    assert!(class(PlanetClass::Lava) > 0, "{classes:?}");
    assert!(
        class(PlanetClass::Island) + class(PlanetClass::Ocean) > 0,
        "{classes:?}"
    );
    assert_eq!(class(PlanetClass::GasGiant), 0);
}

/// Looking up a moon builds one system and the moons of one planet: tens of microseconds.
#[test]
fn moon_lookup_is_cheap() {
    let u = Universe::new(3);
    let home = u.home();
    let last = u.system(home.star).planets.len() as u8 - 1;
    let id = BodyId {
        index: last,
        moon: 1,
        ..home
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
    assert!(best.as_micros() < 60, "a moon lookup took {best:?}");
}
