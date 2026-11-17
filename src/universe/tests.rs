//! Universe tests: the home system is fixed, deterministic and consistent with its own orbits.

use super::*;

/// The same seed gives the same home system, and the home planet matches the game constants.
#[test]
fn home_system_is_deterministic() {
    let (a, b) = (Universe::new(7), Universe::new(7));
    let home = a.home();
    assert_eq!(a.system(home.star), b.system(home.star));
    let planet = a.planet(home);
    assert_eq!(planet.id, home);
    assert_eq!(planet.seed, 7);
    assert_eq!(
        (planet.radius, planet.gravity, planet.day_seconds),
        (HOME_RADIUS, HOME_GRAVITY, HOME_DAY_SECONDS)
    );
    let system = a.system(home.star);
    assert_eq!(system.planets.len(), 4);
    assert!(
        system
            .planets
            .iter()
            .enumerate()
            .all(|(i, p)| usize::from(p.id.index) == i)
    );
}

/// The home galaxy and star are found through the lookups, and nothing else exists yet.
#[test]
fn lookups_find_the_home_star() {
    let u = Universe::new(3);
    let star = u.home().star;
    assert!(u.galaxy(star.galaxy).is_some());
    assert_eq!(u.galaxies_near(star.galaxy, 2).len(), 1);
    let stars = u.stars_in_sector(star.galaxy, star.sector);
    assert_eq!(stars.len(), 1);
    assert_eq!(stars[0].id, star);
    assert!(u.stars_in_sector(star.galaxy, IVec3::ONE).is_empty());
}

/// Planets stay on their orbit radius and the orbit period follows the mass of the star.
#[test]
fn orbits_follow_the_star_mass() {
    let u = Universe::new(1);
    let system = u.system(u.home().star);
    for planet in &system.planets {
        let at = u.planet_position(planet.id, 12_345.0);
        assert!((at.length() - planet.orbit_radius).abs() < 1e-3);
        let back = u.planet_position(planet.id, 12_345.0 + planet.orbit_seconds);
        assert!(at.distance(back) < 1.0);
        let speed = std::f64::consts::TAU * planet.orbit_radius / planet.orbit_seconds;
        let circular = (system.star.mu() / planet.orbit_radius).sqrt();
        assert!((speed / circular - 1.0).abs() < 1e-9);
    }
}
