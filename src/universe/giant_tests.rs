//! Gas giant tests: how many there are, where they form, and that solid planets keep their values.

use super::giant::FROST;
use super::hash::{self, Rng};
use super::range_tests::sample;
use super::*;

/// Returns every system of the star sample but the home systems, each with its star.
pub(super) fn systems() -> Vec<SystemSpec> {
    let free = sample().into_iter().filter(|(u, s)| s.id != u.home().star);
    free.map(|(u, star)| u.system(star.id)).collect()
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

/// Gas giants are 12 to 25 percent of all planets, more common around heavier stars. Shares are
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
    assert!((12.0..=25.0).contains(&all), "{all}");
    assert!(share(&systems, |s| s.class == StarClass::M) > 0.0);
}

/// Gas giants form beyond the frost line, so nearly all lie in the outer half of their system
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
    assert!(outer as f64 >= 0.95 * giants as f64, "{outer} of {giants}");
    assert!(
        warm * 100 <= systems.len(),
        "{warm} in {} systems",
        systems.len()
    );
}

/// A planet that stays solid keeps the radius, gravity, day, orbit and record its first draws
/// gave it before gas giants existed: the giant draws come from a stream of their own.
#[test]
fn solid_planets_keep_their_draws() {
    for system in systems().iter().take(800) {
        let star = &system.star;
        for p in system.planets.iter().filter(|p| !giant(p)) {
            let mut rng = Rng::new(p.seed);
            let size = rng.unit();
            let span = PLANET_RADIUS.1 / PLANET_RADIUS.0;
            assert_eq!(p.radius, PLANET_RADIUS.0 * span.powf(size));
            let earths = planet::earths(size, rng.unit());
            assert_eq!(p.gravity, earths * HOME_GRAVITY);
            assert_eq!(p.day_seconds, rng.range(PLANET_DAY.0, PLANET_DAY.1));
            assert_eq!(p.orbit_phase, rng.range(0.0, std::f64::consts::TAU));
            let draws = hash::child(p.seed, 1);
            let size = p.radius / HOME_RADIUS;
            let record = PlanetRecord::derive(star, p.orbit_radius, earths, size, draws);
            assert_eq!(p.record, record);
        }
    }
}

/// Every gas giant has its radius, gravity, air and cloud deck record inside their ranges.
#[test]
fn giants_stay_in_range() {
    let within = |x: f64, range: (f64, f64)| range.0 <= x && x <= range.1;
    for system in systems() {
        for p in system.planets.iter().filter(|p| giant(p)) {
            let r = p.record;
            assert!(within(p.radius, GIANT_RADIUS), "{p:?}");
            assert!(within(p.gravity / HOME_GRAVITY, GIANT_GRAVITY), "{p:?}");
            assert_eq!(
                [p.atmosphere_height, p.cloud_base, p.cloud_top],
                air(p.radius)
            );
            assert_eq!(r.flux, zone::flux(system.star.luminosity, p.orbit_radius));
            assert_eq!(
                (r.pressure, r.ocean, r.albedo),
                (air::PRESSURE_CAP, 0.0, 0.34)
            );
            assert!(within(r.flux, (0.005, FROST)), "{r:?}");
            assert!(within(r.temperature, (140.0, 260.0)), "{r:?}");
            let wet = climate::state(r.temperature, r.pressure, 1.0);
            assert!(r.water == Water::None || r.water == wet, "{r:?}");
            assert!(r.water != Water::Liquid, "{r:?}");
        }
    }
}
