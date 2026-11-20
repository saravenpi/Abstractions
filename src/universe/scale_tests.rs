//! Scale tests: the galaxy, its stars and the home search at real size, against the numbers of
//! the Milky Way and the neighbourhood of the Sun.

use std::time::Instant;

use bevy::math::{DVec3, IVec3};

use super::galaxy::{self, Shape};
use super::hash::Rng;
use super::home::HOME_GALAXY;
use super::sector;
use super::*;

/// Returns the home star of seed `seed` with its universe and its place in metres.
fn home_star(seed: u64) -> (Universe, DVec3) {
    let u = Universe::new(seed);
    let star = u.star(u.home().star).expect("the home star exists");
    (u, star.pos)
}

/// The light-year and the parsec are the exact products of their definitions, a planet at 1 AU
/// around one Sun takes 365.25 days, and the Earth pulls with 9.81 m/s^2.
#[test]
fn constants_follow_their_definitions() {
    assert!((LY / (SPEED_OF_LIGHT * 31_557_600.0) - 1.0).abs() < 1e-15);
    assert!((PARSEC / (AU * 648_000.0 / std::f64::consts::PI) - 1.0).abs() < 1e-12);
    let year = orbit_seconds(GRAVITATION * SUN_MASS, AU);
    assert!((year / (365.25 * 86_400.0) - 1.0).abs() < 0.001, "{year}");
    let pull = gravity(EARTH_RADIUS, 4050.0);
    assert!((pull / 9.81 - 1.0).abs() < 0.005, "{pull}");
    assert!(
        (EARTH_MASS * GRAVITATION / (STANDARD_GRAVITY * EARTH_RADIUS.powi(2)) - 1.0).abs() < 0.01
    );
}

/// At 0.52 radii out in the plane, between two arms, a sector holds 4 stars on average, and the
/// 10 light-years around the home star hold 10 to 40 stars for 95 percent of 50 seeds.
#[test]
fn the_neighbourhood_of_the_home_has_the_density_of_the_sun() {
    for seed in 0..5 {
        let shape = galaxy::shape(seed, HOME_GALAXY).expect("the home galaxy exists");
        let middle = (shells::sun_point(&shape) / SECTOR_M).floor().as_ivec3();
        let mut stars = Vec::new();
        for i in 0..400 {
            let at = middle + IVec3::new(i % 20 - 10, 0, i / 20 - 10);
            sector::fill(&shape, HOME_GALAXY, at, &mut stars);
        }
        let mean = stars.len() as f64 / 400.0;
        assert!(
            (3.0..5.0).contains(&mean),
            "seed {seed}: {mean} stars per sector"
        );
    }
    let fits = (0..50)
        .map(home_star)
        .filter(|(u, at)| (11..=41).contains(&u.stars_near(HOME_GALAXY, *at, 10.0).len()))
        .count();
    assert!(fits >= 48, "{fits} of 50 seeds");
}

/// The star nearest to the home star lies 1 to 10 light-years away for 95 percent of 200 seeds.
#[test]
fn the_nearest_star_is_a_few_light_years_away() {
    let fits = (0..200)
        .map(home_star)
        .filter(|(u, at)| {
            let near = u.stars_near(HOME_GALAXY, *at, 10.0);
            near.get(1).is_some_and(|s| s.pos.distance(*at) >= LY)
        })
        .count();
    assert!(fits >= 190, "{fits} of 200 seeds");
}

/// Neighbour galaxies lie 1.0 to 4.5 million light-years apart for 95 percent of the pairs of 100
/// universes, the Local Group having 3 million between its two big spirals.
#[test]
fn galaxies_lie_millions_of_light_years_apart() {
    let (mut pairs, mut fits) = (0, 0);
    for seed in 0..100 {
        let u = Universe::new(seed);
        let home = u.galaxy(HOME_GALAXY).expect("the home galaxy exists");
        for step in [IVec3::X, IVec3::Y, IVec3::Z] {
            let Some(next) = u.galaxy(GalaxyId(step)) else {
                continue;
            };
            let apart = step.as_dvec3() * GALAXY_CELL + next.center - home.center;
            let mly = apart.length() / LY / 1.0e6;
            pairs += 1;
            fits += usize::from((1.0..=4.5).contains(&mly));
        }
    }
    assert!(
        pairs > 100 && fits * 20 >= pairs * 19,
        "{fits} of {pairs} pairs"
    );
}

/// A spiral 50,000 light-years in radius holds 30 billion to 400 billion stars, the Milky Way
/// 100 to 400 billion: a Monte Carlo of 20,000 sectors over the box that holds its disc and core.
#[test]
fn a_spiral_holds_as_many_stars_as_the_milky_way() {
    let radius = 50_000.0;
    let shape = Shape {
        radius,
        ..Shape::new(1, true)
    };
    let (half, tall) = (radius / SECTOR_LY, 0.35 * radius / SECTOR_LY);
    let mut rng = Rng::new(7);
    let mut sum = 0.0;
    for _ in 0..20_000 {
        let at = DVec3::new(
            rng.range(-half, half),
            rng.range(-tall, tall),
            rng.range(-half, half),
        );
        sum += shape
            .parts(at * SECTOR_LY)
            .total()
            .min(f64::from(SECTOR_STARS));
    }
    let total = sum / 20_000.0 * (2.0 * half) * (2.0 * half) * (2.0 * tall);
    println!("stars of a spiral of {radius} light-years: {total:.3e}");
    assert!((3.0e10..4.0e11).contains(&total), "{total:e}");
}

/// Choosing the home takes under 2 seconds for 20 seeds together.
#[test]
fn the_home_search_stays_under_two_seconds() {
    let start = Instant::now();
    for seed in 0..20 {
        std::hint::black_box(Universe::new(seed));
    }
    let took = start.elapsed();
    println!("20 home searches: {took:?}");
    assert!(took.as_secs_f64() < 2.0, "{took:?}");
}

/// No two of 2,000 stars of the home neighbourhood (60 light-years around it, the home lying
/// anywhere on the disc) and of a core block lie closer than their
/// two system reaches, so no sky ever has two systems in one.
#[test]
fn stars_keep_out_of_each_others_systems() {
    let (u, at) = home_star(1);
    let galaxy = u.galaxy(HOME_GALAXY).expect("the home galaxy exists");
    let core = DVec3::new(0.04, 0.0, 0.03) * galaxy.radius;
    let mut stars = u.stars_near(HOME_GALAXY, at, 60.0);
    stars.extend(u.stars_near(HOME_GALAXY, core, 15.0));
    stars.truncate(2000);
    assert!(stars.len() >= 1000, "{}", stars.len());
    for (i, a) in stars.iter().enumerate() {
        for b in &stars[i + 1..] {
            let apart = a.pos.distance(b.pos);
            assert!(apart > system_reach(a) + system_reach(b), "{a:?} {b:?}");
        }
    }
}
