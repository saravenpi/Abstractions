//! Gas giant tests: how many there are, where they form, and that solid planets keep their values.

use super::hash::{self, Rng};
use super::range_tests::sample;
use super::record::Body;
use super::spin::{FASTEST, SLOWEST, SLOWEST_GIANT};
use super::volatiles::FROST;
use super::*;

/// Returns every system of the star sample, each with its star.
pub(super) fn systems() -> Vec<SystemSpec> {
    let stars = sample().into_iter();
    stars.map(|(u, star)| u.system(star.id)).collect()
}

/// Returns true when a planet is a gas giant.
pub(super) fn giant(planet: &PlanetSpec) -> bool {
    planet.record.class == PlanetClass::GasGiant
}

/// Returns the percentage of gas giants among the planets of the systems that pass `keep`.
fn share(systems: &[SystemSpec], keep: impl Fn(&StarSpec) -> bool) -> f64 {
    let kept = systems.iter().filter(|s| keep(&s.star));
    let (giants, total) = kept.fold((0, 0), |(g, t), s| {
        let found = s.planets.iter().filter(|p| giant(p)).count();
        (g + found, t + s.planets.len())
    });
    (1000.0 * giants as f64 / total as f64).round() / 10.0
}

/// Gas giants are 15 to 40 percent of all planets, more common around heavier stars. Shares are
/// printed per star class.
#[test]
fn gas_giants_have_their_share() {
    let systems = systems();
    let all = share(&systems, |_| true);
    println!("gas giants, percent of planets: all {all}");
    let classes = [
        StarClass::A,
        StarClass::F,
        StarClass::G,
        StarClass::K,
        StarClass::M,
    ];
    for class in classes {
        println!("{class:?} {}", share(&systems, |s| s.class == class));
    }
    assert!((15.0..=40.0).contains(&all), "{all}");
    let (red, yellow) = (
        share(&systems, |s| s.class == StarClass::M),
        share(&systems, |s| s.class == StarClass::G),
    );
    assert!(red > 0.0 && yellow > red, "{red} {yellow}");
}

/// Gas giants form beyond the frost line, so nine in ten lie in the outer half of their system
/// and at most one in a hundred systems has one in its habitable zone.
#[test]
fn gas_giants_are_outer() {
    let systems = systems();
    let (mut giants, mut outer, mut warm) = (0, 0, 0);
    for system in &systems {
        let count = system.planets.len();
        for p in system.planets.iter().filter(|p| giant(p)) {
            assert!(p.record.flux < FROST, "{p:?}");
            giants += 1;
            outer += usize::from(2 * usize::from(p.id.index) >= count);
            warm += usize::from(p.record.zone == Zone::Habitable);
        }
    }
    println!("{giants} giants, {outer} in the outer half, {warm} in a habitable zone");
    assert!(outer as f64 >= 0.9 * giants as f64, "{outer} of {giants}");
    assert!(
        warm * 100 <= systems.len(),
        "{warm} in {} systems",
        systems.len()
    );
}

/// A planet that stays solid has the radius, gravity, spin, orbit phase and record its own
/// draws give it: the giant draws come from a stream of their own.
#[test]
fn solid_planets_keep_their_draws() {
    for system in systems().iter().take(800) {
        let star = &system.star;
        for p in system.planets.iter().filter(|p| !giant(p)) {
            let mut rng = Rng::new(p.seed);
            let size = 0.5 * (rng.unit() + rng.unit());
            let span = PLANET_RADIUS.1 / PLANET_RADIUS.0;
            assert_eq!(p.radius, PLANET_RADIUS.0 * span.powf(size));
            assert_eq!(p.gravity, gravity(p.radius, rng.range(ROCK.0, ROCK.1)));
            let born = spin::born(rng.unit(), p.radius, p.gravity, SLOWEST);
            assert!(p.day_seconds == born || p.locked(), "{p:?}");
            assert_eq!(p.orbit_phase, rng.range(0.0, std::f64::consts::TAU));
            let body = Body::new(p.gravity, p.radius, p.orbit_radius);
            let record = PlanetRecord::derive(star, &body, hash::child(p.seed, 1));
            assert_eq!(p.record, record);
            assert_eq!(p.record.tide, 0.0);
        }
    }
}

/// The air heights of every gas giant are those its record gives them at its cloud deck, and
/// its spin lies between the fastest its size allows and the slowest a giant is born with.
#[test]
fn giants_have_the_air_and_spin_their_size_gives() {
    let within = |x: f64, range: (f64, f64)| range.0 <= x && x <= range.1;
    for system in systems() {
        for p in system.planets.iter().filter(|p| giant(p)) {
            let fastest = FASTEST * breakup(p.radius, p.gravity);
            assert!(p.locked() || within(p.day_seconds, (fastest, SLOWEST_GIANT)));
            assert_eq!(
                [p.atmosphere_height, p.cloud_base, p.cloud_top],
                heights(&p.record, p.gravity, p.radius)
            );
        }
    }
}

/// Every gas giant has its radius, gravity, spin, air and cloud deck record inside their ranges.
#[test]
fn giants_stay_in_range() {
    let within = |x: f64, range: (f64, f64)| range.0 <= x && x <= range.1;
    for system in systems() {
        for p in system.planets.iter().filter(|p| giant(p)) {
            let r = p.record;
            assert!(within(p.radius, GIANT_RADIUS), "{p:?}");
            let mass = p.mu() / GRAVITATION;
            let span = (
                GIANT_MASS.0 / giant::MASS_SPREAD,
                GIANT_MASS.1 * giant::MASS_SPREAD,
            );
            assert!(
                within(mass, span) && within(p.gravity, (4.8, 52.0)),
                "{p:?}"
            );
            assert_eq!(r.flux, zone::flux(system.star.luminosity, p.orbit_radius));
            assert_eq!(
                (r.pressure, r.ocean, r.albedo),
                (air::PRESSURE_CAP, 0.0, 0.34)
            );
            assert_eq!((r.ammonia, r.hydrocarbon, r.tide), (0.0, 0.0, 0.0));
            assert!(
                r.flux < FROST && within(r.temperature, (115.0, 265.0)),
                "{r:?}"
            );
            let wet = climate::state(r.temperature, r.pressure, 1.0);
            assert!(r.water == Water::None || r.water == wet, "{r:?}");
            assert!(r.water != Water::Liquid, "{r:?}");
        }
    }
}
