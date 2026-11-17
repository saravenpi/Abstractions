//! Universe tests: the home system is pinned, everything is deterministic and the lookups agree.

use std::time::Instant;

use bevy::math::DVec3;

use super::*;

/// The same seed gives the same home system, and the home planet matches the game constants.
#[test]
fn home_system_is_deterministic() {
    let (a, b) = (Universe::new(7), Universe::new(7));
    let home = a.home();
    assert_eq!(home, b.home());
    assert_eq!(a.system(home.star), b.system(home.star));
    let planet = a.planet(home);
    assert_eq!(planet.id, home);
    assert_eq!(planet.seed, 7);
    assert_eq!(
        (planet.radius, planet.gravity, planet.day_seconds),
        (HOME_RADIUS, HOME_GRAVITY, HOME_DAY_SECONDS)
    );
    let air = (
        planet.atmosphere_height,
        planet.cloud_base,
        planet.cloud_top,
    );
    assert_eq!(air, (4000.0, 900.0, 1500.0));
    let system = a.system(home.star);
    assert!((3..=8).contains(&system.planets.len()));
    assert!(
        system
            .planets
            .iter()
            .enumerate()
            .all(|(i, p)| usize::from(p.id.index) == i)
    );
}

/// Every seed has a pinned home planet in the disc of a spiral home galaxy.
#[test]
fn every_seed_has_a_home_in_the_disc() {
    for seed in 0..200 {
        let u = Universe::new(seed);
        let home = u.home();
        let galaxy = u
            .galaxy(home.star.galaxy)
            .expect("the home cell holds a galaxy");
        assert_eq!(galaxy.kind, GalaxyKind::Spiral);
        let star = u.star(home.star).expect("the home star exists");
        assert_eq!(
            (star.class, star.luminosity, star.id),
            (StarClass::G, 1.0, home.star)
        );
        let reach = star.pos.x.hypot(star.pos.z) / galaxy.radius;
        assert!((0.39..0.61).contains(&reach), "seed {seed}: {reach}");
        assert!(star.pos.y.abs() <= SECTOR_GLY * GLY);
        let planet = u.planet(home);
        assert_eq!(
            (planet.id, planet.seed, planet.radius),
            (home, seed, HOME_RADIUS)
        );
        assert_eq!(
            (planet.gravity, planet.day_seconds),
            (HOME_GRAVITY, HOME_DAY_SECONDS)
        );
        assert!(!planet.name.starts_with(&star.name));
    }
}

/// The home galaxy and star are found through the lookups, in index order, and a missing star is nothing.
#[test]
fn lookups_find_the_home_star() {
    let u = Universe::new(3);
    let star = u.home().star;
    assert!(u.galaxy(star.galaxy).is_some());
    let near = u.galaxies_near(star.galaxy, 2);
    assert!(near.iter().any(|g| g.id == star.galaxy));
    assert!(
        near.iter()
            .all(|g| (g.id.0 - star.galaxy.0).abs().max_element() <= 2)
    );
    assert_eq!(u.galaxies_near(star.galaxy, 0).len(), 1);
    let stars = u.stars_in_sector(star.galaxy, star.sector);
    assert_eq!(stars[0].id, star);
    assert!(
        stars
            .iter()
            .enumerate()
            .all(|(i, s)| usize::from(s.id.index) == i)
    );
    assert!(
        u.stars_in_sector(star.galaxy, IVec3::splat(1000))
            .is_empty()
    );
    assert_eq!(u.star(StarId { index: 200, ..star }), None);
    assert_eq!(u.system(StarId { index: 200, ..star }), u.system(star));
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

/// The same seed gives the same galaxies and stars twice, and another seed gives other ones.
#[test]
fn seeds_decide_everything() {
    let (a, b, c) = (Universe::new(11), Universe::new(11), Universe::new(12));
    let home = a.home().star;
    assert_eq!(
        a.galaxies_near(home.galaxy, 2),
        b.galaxies_near(home.galaxy, 2)
    );
    assert_ne!(
        a.galaxies_near(home.galaxy, 2),
        c.galaxies_near(home.galaxy, 2)
    );
    let at = a.star(home).expect("the home star exists").pos;
    let (near, again) = (
        a.stars_near(home.galaxy, at, 30.0),
        b.stars_near(home.galaxy, at, 30.0),
    );
    assert!(near.len() > 3);
    assert_eq!(near, again);
    assert_ne!(near, c.stars_near(home.galaxy, at, 30.0));
    for star in &near {
        assert_eq!(a.system(star.id), b.system(star.id));
    }
    assert_ne!(a.system(home), c.system(c.home().star));
}

/// Near stars come nearest first, inside the radius, and each is found again by its identifier.
#[test]
fn near_stars_are_sorted_and_found_again() {
    let u = Universe::new(5);
    let home = u.home().star;
    let galaxy = u.galaxy(home.galaxy).expect("the home cell holds a galaxy");
    let at = u.star(home).expect("the home star exists").pos;
    let near = u.stars_near(home.galaxy, at, 60.0);
    assert_eq!(near[0].id, home);
    assert!(near.len() > 10, "only {} stars within 60 gly", near.len());
    assert!(
        near.windows(2)
            .all(|w| w[0].pos.distance(at) <= w[1].pos.distance(at))
    );
    for star in &near {
        assert!(star.pos.distance(at) <= 60.0 * GLY);
        assert_eq!(sector_of(star.pos), star.id.sector);
        assert_eq!(u.star(star.id).as_ref(), Some(star));
        let pos = u.star_position(star.id).expect("the star exists");
        assert_eq!(pos, galaxy.locate(star.pos));
        assert_eq!(pos.cell, home.galaxy.0);
    }
    assert!(
        u.stars_near(GalaxyId(IVec3::splat(i32::MAX)), at, 0.0)
            .len()
            <= 1
    );
}

/// The density has a void at the centre, a dense core, a thin disc and nothing far outside.
#[test]
fn density_follows_the_galaxy_shape() {
    let u = Universe::new(9);
    let galaxy = u
        .galaxy(u.home().star.galaxy)
        .expect("the home cell holds a galaxy");
    let r = galaxy.radius;
    assert_eq!(galaxy.density(DVec3::ZERO), 0.0);
    assert_eq!(galaxy.density(DVec3::X * r * 1.5), 0.0);
    let core = galaxy.density(DVec3::X * r * 0.03);
    let ring = |reach: f64, up: f64| -> f64 {
        let at =
            |i: i32| DVec3::new(f64::from(i).cos() * reach, up, f64::from(i).sin() * reach) * r;
        (0..64).map(|i| galaxy.density(at(i))).sum::<f64>() / 64.0
    };
    assert!(core > 1.0 && core > 4.0 * ring(0.5, 0.0));
    assert!(ring(0.3, 0.0) > ring(0.6, 0.0) && ring(0.6, 0.0) > ring(0.95, 0.0));
    assert!(ring(0.5, 0.0) > 20.0 * ring(0.5, 0.2));
    assert!(ring(0.5, 0.3) > 0.0);
}

/// One thousand sectors take about 0.3 ms, far under the 50 ms budget: 5 ms still catches a tenfold slowdown.
#[test]
fn sectors_generate_fast() {
    let u = Universe::new(2);
    let home = u.home().star;
    let run = || {
        let start = Instant::now();
        let offsets = (0..1000).map(|i| IVec3::new(i % 10, i / 10 % 10, i / 100) - 5);
        let stars: usize = offsets
            .map(|d| u.stars_in_sector(home.galaxy, home.sector + d).len())
            .sum();
        (start.elapsed(), stars)
    };
    let (best, stars) = (0..5).map(|_| run()).min().expect("five runs");
    println!("1000 sectors, {stars} stars, {best:?}");
    assert!(stars > 20);
    assert!(best.as_millis() < 5, "1000 sectors took {best:?}");
}
