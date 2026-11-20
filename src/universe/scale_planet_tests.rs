//! Scale tests for planets: one Hill radius formula, the star laws against their sources, the
//! spacing of planets and the Roche bound of rings.

use super::giant_tests::systems;
use super::range_tests::sample;
use super::rings::{DUST_DENSITY, ICE_DENSITY, roche};
use super::system::SPACING;
use super::volatiles::FROST;
use super::*;

/// A planet's own Hill radius and the one of the formula the moons and the spacing use agree to
/// a billionth for 200 planets.
#[test]
fn the_hill_radius_has_one_formula() {
    let both = sample().into_iter().flat_map(|(u, star)| {
        let planets = u.system(star.id).planets;
        planets.into_iter().map(move |p| (star.clone(), p))
    });
    let mut count = 0;
    for (star, p) in both.take(200) {
        let by_star = hill::hill(&star, &p);
        assert!((p.hill_radius() / by_star - 1.0).abs() < 1e-9, "{p:?}");
        count += 1;
    }
    assert_eq!(count, 200);
}

/// The mass-luminosity law gives 0.0057 Suns at 0.2, 5.06 at 1.5 and 4,470 at 10 Suns, and the
/// water frost line of a star of luminosity 1 lies at 2.67 astronomical units.
#[test]
fn star_laws_match_their_sources() {
    let close = |x: f64, y: f64| (x / y - 1.0).abs() < 0.01;
    assert!(close(star::light(0.2), 0.23 * 0.2f64.powf(2.3)));
    assert!(close(star::light(0.2), 0.0057));
    assert!(close(star::light(1.5), 5.06));
    assert!(close(star::light(10.0), 4470.0));
    assert!(close((1.0 / FROST).sqrt() * AU, 2.67 * AU));
}

/// Every ring ends inside the Roche limit of its own matter: ice or dust.
#[test]
fn rings_end_inside_the_roche_limit_of_their_matter() {
    let mut rings = 0;
    for system in systems() {
        for p in system.planets.iter().filter(|p| p.rings.is_some()) {
            let matter = if p.record.temperature < 220.0 {
                ICE_DENSITY
            } else {
                DUST_DENSITY
            };
            let outer = p.rings.map_or(0.0, |r| r.outer);
            assert!(outer <= roche(p, matter) * p.radius * (1.0 + 1e-9), "{p:?}");
            rings += 1;
        }
    }
    assert!(rings > 20, "{rings}");
}

/// Neighbouring planets keep 2 sqrt(3) mutual Hill radii between their orbits, and the first
/// orbit lies at least 3 star radii out.
#[test]
fn planets_keep_their_mutual_hill_spacing() {
    for (u, star) in sample() {
        let planets = u.system(star.id).planets;
        assert!(planets[0].orbit_radius > 3.0 * star.radius, "{star:?}");
        for pair in planets.windows(2) {
            let pull = (pair[0].mu() + pair[1].mu()) / (3.0 * star.mu());
            let mutual = pull.cbrt() * 0.5 * (pair[0].orbit_radius + pair[1].orbit_radius);
            let gap = pair[1].orbit_radius - pair[0].orbit_radius;
            assert!(gap >= SPACING * mutual * (1.0 - 1e-6), "{pair:?}");
        }
    }
}
