//! Spin and orbit tests: real sizes give real gravities, days and years, tides lock the close
//! bodies, and no orbit crowds another.

use super::giant_tests::systems;
use super::hill::hill;
use super::range_tests::sample;
use super::spin::{FASTEST, QUALITY, SLOWEST, Turning, born, lock_seconds, settled};
use super::*;

/// The Earth as tides see it: its radius, its gravity and its 86,164 s turn.
const EARTH: Turning = Turning {
    radius: EARTH_RADIUS,
    gravity: STANDARD_GRAVITY,
    day: 86_164.0,
    quality: QUALITY.0,
};

/// A planet with the size and the rock of the Earth has its gravity, its break-up period of 84
/// minutes, its year at one astronomical unit from a star like the Sun, and the orbital and
/// escape speeds that follow.
#[test]
fn an_earth_twin_has_the_values_of_the_earth() {
    let pull = gravity(EARTH_RADIUS, 4050.0);
    assert!((pull / STANDARD_GRAVITY - 1.0).abs() < 0.005, "{pull}");
    assert!((breakup(EARTH_RADIUS, STANDARD_GRAVITY) / 60.0 - 84.4).abs() < 0.2);
    let year = orbit_seconds(GRAVITATION * SUN_MASS, AU);
    assert!((year / YEAR - 1.0).abs() < 0.001, "{year}");
    let mu = STANDARD_GRAVITY * EARTH_RADIUS * EARTH_RADIUS;
    let orbital = (mu / EARTH_RADIUS).sqrt();
    assert!((orbital - 7905.0).abs() < 10.0, "{orbital}");
    assert!((2f64.sqrt() * orbital - 11_180.0).abs() < 15.0);
    assert!((1.2..1.4).contains(&gravity(PLANET_RADIUS.0, ROCK.0)));
    assert!((40.0..42.0).contains(&gravity(PLANET_RADIUS.1, ROCK.1)));
    assert!((3.5..4.0).contains(&gravity(3.39e6, 3700.0)), "Mars");
}

/// Tides take about 30 billion years to lock the Earth and under a billion to lock a planet at
/// the distance of Mercury, and a planet in the habitable zone of a red dwarf is locked within
/// a million years. A locked body takes its year for a day.
#[test]
fn tides_lock_close_planets() {
    let sun = GRAVITATION * SUN_MASS;
    let earth = lock_seconds(&EARTH, sun, AU) / YEAR;
    assert!((2.0e10..4.0e10).contains(&earth), "{earth}");
    let close = lock_seconds(&EARTH, sun, 0.387 * AU) / YEAR;
    assert!(close < 1.5e8, "{close}");
    let dwarf = lock_seconds(&EARTH, 0.2 * sun, 0.075 * AU) / YEAR;
    assert!(dwarf < 1.0e6, "{dwarf}");
    let age = 4.6e9 * YEAR;
    assert_eq!(settled(&EARTH, (sun, AU, YEAR), age), EARTH.day);
    let year = orbit_seconds(sun, 0.387 * AU);
    assert_eq!(settled(&EARTH, (sun, 0.387 * AU, year), age), year);
}

/// A body is born turning between twice its break-up period and its slowest day, on a log scale.
#[test]
fn born_spins_stay_in_range() {
    let fastest = FASTEST * breakup(EARTH_RADIUS, STANDARD_GRAVITY);
    assert_eq!(born(0.0, EARTH_RADIUS, STANDARD_GRAVITY, SLOWEST), fastest);
    let slowest = born(1.0, EARTH_RADIUS, STANDARD_GRAVITY, SLOWEST);
    assert!((slowest / SLOWEST - 1.0).abs() < 1e-12);
    let middle = born(0.5, EARTH_RADIUS, STANDARD_GRAVITY, SLOWEST);
    assert!((middle / (fastest * SLOWEST).sqrt() - 1.0).abs() < 1e-12);
    assert_eq!(born(0.7, 7.0e7, 0.1, 10.0), FASTEST * breakup(7.0e7, 0.1));
}

/// Every planet and moon of the sample turns slowly enough: its equator moves at under half
/// the speed of a low orbit. A planet is locked to its star or keeps the spin it was born with,
/// and more planets are locked around red dwarfs than around yellow stars.
#[test]
fn spins_are_physical() {
    let (mut red, mut yellow) = ((0.0, 0.0), (0.0, 0.0));
    for system in systems() {
        for p in &system.planets {
            let bodies = std::iter::once(p.clone()).chain(moon::of(&system.star, p));
            for body in bodies {
                let equator = std::f64::consts::TAU * body.radius / body.day_seconds;
                let orbital = (body.gravity * body.radius).sqrt();
                assert!(equator > 0.0 && equator <= 0.5 * orbital * (1.0 + 1e-9));
            }
            let side = match system.star.class {
                StarClass::M => &mut red,
                StarClass::G => &mut yellow,
                _ => continue,
            };
            *side = (side.0 + f64::from(u8::from(p.locked())), side.1 + 1.0);
        }
    }
    let (red, yellow) = (red.0 / red.1, yellow.0 / yellow.1);
    println!("planets locked to their star: red dwarfs {red:.2}, yellow stars {yellow:.2}");
    assert!(red > yellow && red > 0.3 && yellow < 0.6, "{red} {yellow}");
}

/// No orbit lies inside its star, and no planet's Hill sphere reaches the Hill sphere of a
/// neighbour.
#[test]
fn orbits_clear_each_other() {
    for (u, star) in sample() {
        let planets = u.system(star.id).planets;
        let reach = |p: &PlanetSpec| hill(&star, p);
        assert!(planets[0].orbit_radius - reach(&planets[0]) > star.radius);
        for pair in planets.windows(2) {
            let (inner, outer) = (&pair[0], &pair[1]);
            let gap = outer.orbit_radius - inner.orbit_radius;
            assert!(gap > reach(inner) + reach(outer), "{inner:?} {outer:?}");
        }
    }
}
