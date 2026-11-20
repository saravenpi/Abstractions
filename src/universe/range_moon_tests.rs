//! Moon range tests: every moon of the sample has its size, pull, day and record inside the
//! ranges its items document.

use super::moon_tests::hosts;
use super::*;

/// Returns true when `x` lies between both ends of a range, ends included.
fn within(x: f64, range: (f64, f64)) -> bool {
    range.0 <= x && x <= range.1
}

/// Every moon of the sample has the radius of one of the three kinds of moon, the gravity its
/// size and matter give it, a day as long as its orbit and a record inside its ranges.
#[test]
fn moons_stay_in_range() {
    for (star, p, moons) in hosts() {
        for m in moons {
            let r = m.record;
            let impact = (IMPACT_RADIUS.0 * p.radius, IMPACT_RADIUS.1 * p.radius);
            let sized = [impact, CAPTURED_RADIUS, REGULAR_RADIUS];
            assert!(sized.iter().any(|range| within(m.radius, *range)), "{m:?}");
            let pull = (gravity(m.radius, 1500.0), gravity(m.radius, 3500.0));
            assert!(within(m.gravity, pull), "{m:?}");
            assert!(within(m.orbit_phase, (0.0, std::f64::consts::TAU)));
            assert!(m.day_seconds > 2.0 * breakup(m.radius, m.gravity), "{m:?}");
            assert_eq!(
                [m.atmosphere_height, m.cloud_base, m.cloud_top],
                air(m.radius)
            );
            assert_eq!(r.flux, zone::flux(star.luminosity, p.orbit_radius));
            assert!(within(r.pressure, (0.0, air::PRESSURE_CAP)), "{r:?}");
            assert!(within(r.albedo, (0.08, 0.6)) && within(r.greenhouse, (0.0, 1500.0)));
            assert!(within(r.temperature, (5.0, 7000.0)) && within(r.ocean, (0.0, 1.0)));
            assert_eq!(r.class, class::classify(&r));
        }
    }
}

/// Tides heat only the regular moons of gas giants, 2 to the power 7.5 times less at twice the
/// distance, and a moon placed like Io around a planet like Jupiter gets the heat of Io. The
/// hottest moons are lava worlds, air or not, and no moon is a gas giant.
#[test]
fn tides_heat_close_moons() {
    let mut classes = [0u32; 7];
    for (_, p, moons) in hosts() {
        let giant = p.record.class == PlanetClass::GasGiant;
        for m in moons {
            classes[m.record.class as usize] += 1;
            let regular = giant && m.radius >= REGULAR_RADIUS.0;
            assert!(regular || m.record.tide == 0.0, "{m:?}");
            assert!((0.0..1.0).contains(&m.record.tide), "{m:?}");
            let volcanic = m.record.tide >= VOLCANIC;
            assert!(!volcanic || m.record.class == PlanetClass::Lava, "{m:?}");
        }
    }
    println!("moon classes, ocean to gas giant: {classes:?}");
    assert!(classes[PlanetClass::Lava as usize] > 0, "{classes:?}");
    assert_eq!(classes[PlanetClass::GasGiant as usize], 0);
    let (_, jupiter, _) = hosts().swap_remove(0);
    let jupiter = PlanetSpec {
        radius: 6.99e7,
        gravity: 1.267e17 / (6.99e7 * 6.99e7),
        ..jupiter
    };
    let io = |orbit: f64| hill::Place {
        orbit,
        radius: 1.8216e6,
        forcing: 1.0,
        ..hill::Place::default()
    };
    let heat = moon::tide(&jupiter, &io(4.217e8));
    assert!((heat / moon::IO_HEAT - 1.0).abs() < 1e-9, "{heat}");
    let further = moon::tide(&jupiter, &io(2.0 * 4.217e8));
    assert!((2f64.powf(7.5) * further / heat - 1.0).abs() < 1e-9);
}

/// Looking one moon up by its id builds the same spec as listing every moon of its planet,
/// for every planet of the sample with moons and every rank, and a rank past the last moon
/// gives the planet. Prints how long one lookup of a moon takes.
#[test]
fn a_moon_looked_up_is_the_moon_listed() {
    let (mut looked, mut time) = (0u32, std::time::Duration::ZERO);
    for (universe, star) in super::range_tests::sample().into_iter().take(40) {
        for planet in universe
            .system(star.id)
            .planets
            .iter()
            .filter(|p| p.moons > 0)
        {
            let moons = universe.moons(planet.id);
            for rank in 1..=planet.moons + 1 {
                let id = BodyId {
                    moon: rank,
                    ..planet.id
                };
                let start = std::time::Instant::now();
                let found = universe.planet(id);
                (looked, time) = (looked + 1, time + start.elapsed());
                let listed = moons.get(usize::from(rank) - 1).unwrap_or(planet);
                assert_eq!(&found, listed, "{id:?}");
            }
        }
    }
    assert!(looked > 20, "only {looked} lookups");
    println!(
        "Universe::planet(moon): {:?} each over {looked}",
        time / looked
    );
}
