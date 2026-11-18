//! Moon range tests: every moon of the sample has its size, pull, day and record inside the
//! ranges its items document.

use super::moon_tests::hosts;
use super::*;

/// Returns true when `x` lies between both ends of a range, ends included.
fn within(x: f64, range: (f64, f64)) -> bool {
    range.0 <= x && x <= range.1
}

/// Every moon of the sample has a radius of 3 to 15 km, inside the range of solid planets, a
/// gravity inside the planet range, a day as long as its orbit, turning the other way, and a record inside its ranges.
#[test]
fn moons_stay_in_range() {
    for (star, p, moons) in hosts() {
        for m in moons {
            let r = m.record;
            assert!(
                within(m.radius, MOON_RADIUS) && within(m.radius, PLANET_RADIUS),
                "{m:?}"
            );
            assert!(within(m.gravity / HOME_GRAVITY, PLANET_GRAVITY), "{m:?}");
            assert!(within(m.orbit_phase, (0.0, std::f64::consts::TAU)));
            assert!(within(-m.day_seconds, (600.0, 250_000.0)), "{m:?}");
            assert_eq!(
                [m.atmosphere_height, m.cloud_base, m.cloud_top],
                air(m.radius)
            );
            assert_eq!(r.flux, zone::flux(star.luminosity, p.orbit_radius));
            assert!(within(r.pressure, (0.0, air::PRESSURE_CAP)), "{r:?}");
            assert!(within(r.albedo, (0.08, 0.6)) && within(r.greenhouse, (0.0, 1500.0)));
            assert!(within(r.temperature, (40.0, 6000.0)) && within(r.ocean, (0.0, 1.0)));
            assert_eq!(r.class, class::classify(&r));
        }
    }
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
