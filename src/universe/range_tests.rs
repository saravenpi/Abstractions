//! Range tests: every generated value lies inside its documented range, over thousands of objects.

use bevy::math::{DVec3, IVec3};

use super::density::Region;
use super::*;

/// Returns at least 2,000 stars: the neighbourhood of several homes and a block of each dense core.
pub(super) fn sample() -> Vec<(Universe, StarSpec)> {
    let mut stars = Vec::new();
    for seed in 0..6 {
        let u = Universe::new(seed);
        let home = u.home().star;
        let galaxy = u.galaxy(home.galaxy).expect("the home cell holds a galaxy");
        let at = u.star(home).expect("the home star exists").pos;
        let core = DVec3::new(0.04, 0.0, 0.03) * galaxy.radius;
        let found = [
            u.stars_near(home.galaxy, at, 60.0),
            u.stars_near(home.galaxy, core, 45.0),
        ];
        stars.extend(found.into_iter().flatten().map(|star| (u, star)));
    }
    assert!(stars.len() >= 2000, "only {} stars sampled", stars.len());
    stars
}

/// Returns true when `x` lies between both ends of a range, ends included.
fn within(x: f64, range: (f64, f64)) -> bool {
    range.0 <= x && x <= range.1
}

/// Every star of the sample has a mass, light, radius, temperature, age, colour and place
/// inside its range, and its light and size follow its mass to a billionth, the rounding of the
/// kilogram round trip.
#[test]
fn stars_stay_in_range() {
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(b.abs());
    for (_, star) in sample() {
        let mass = star.mass / SUN_MASS;
        assert!(within(mass, (0.08, 30.0)), "{star:?}");
        assert!(within(star.luminosity, (0.0006, 2.2e5)), "{star:?}");
        assert!(within(star.radius / SUN_RADIUS, (0.13, 7.0)), "{star:?}");
        assert!(within(star.temperature, (2500.0, 47_000.0)), "{star:?}");
        assert!(close(star.luminosity, star::light(mass)), "{star:?}");
        assert!(
            close(star.radius, star::size(mass) * SUN_RADIUS),
            "{star:?}"
        );
        let years = star.age / YEAR;
        assert!(years > 0.0 && years <= star::OLDEST, "{star:?}");
        assert!(years <= star::life(mass), "{star:?}");
        assert!(star.color.min_element() > 0.0 && star.color.max_element() <= 1.0);
        assert!(usize::from(star.id.index) < usize::from(SECTOR_STARS));
        assert_eq!(sector_of(star.pos), star.id.sector);
        assert!(!star.name.is_empty());
    }
}

/// Every system of the sample has 2 to 8 planets, numbered outward on orbits that grow, none
/// closer than the span of orbits of its star allows and none inside the star.
#[test]
fn systems_stay_in_range() {
    for (u, star) in sample() {
        let system = u.system(star.id);
        let scale = star.mass / SUN_MASS;
        assert_eq!(system.star, star);
        assert!((PLANETS.0..=PLANETS.1).contains(&(system.planets.len() as u8)));
        let numbered = system.planets.iter().zip(0u8..);
        assert!(numbered.clone().all(|(p, index)| p.id.index == index));
        assert!(system.planets.iter().all(|p| p.id.star == star.id));
        let grows = |pair: &[PlanetSpec]| pair[0].orbit_radius < pair[1].orbit_radius;
        assert!(system.planets.windows(2).all(grows), "{star:?}");
        let span = (ORBIT.0 * scale, 3.0 * ORBIT.1 * scale);
        assert!(system.planets.iter().all(|p| within(p.orbit_radius, span)));
        assert!(
            system.planets[0].orbit_radius > 2.0 * star.radius,
            "{star:?}"
        );
    }
}

/// Gravity range of a solid planet and of a gas giant, in metres per second squared.
const GRAVITY: [(f64, f64); 2] = [(1.28, 41.3), (4.8, 52.0)];

/// Every planet of the sample has a radius, gravity, year and air inside its range, those of
/// solid worlds or those of gas giants. Its air holds together: a scale height no planet lets
/// go of, clouds that top out between one scale height and the height where rising air runs
/// out of pressure, and a cloud base above the ground, at most half way up to that top. It has
/// no host and at most 6 moons.
#[test]
fn planets_stay_in_range() {
    for (u, star) in sample() {
        for p in u.system(star.id).planets {
            let giant = p.record.class == PlanetClass::GasGiant;
            let radius = if giant { GIANT_RADIUS } else { PLANET_RADIUS };
            assert!(within(p.radius, radius), "{p:?}");
            assert!(within(p.gravity, GRAVITY[usize::from(giant)]), "{p:?}");
            assert!(within(p.orbit_phase, (0.0, std::f64::consts::TAU)));
            assert_eq!(p.orbit_seconds, orbit_seconds(star.mu(), p.orbit_radius));
            assert_eq!(p.mu(), p.gravity * p.radius * p.radius);
            assert_eq!(
                [p.atmosphere_height, p.cloud_base, p.cloud_top],
                heights(&p.record, p.gravity, p.radius)
            );
            let (scale, slack) = (p.scale_height(), 1.0 + 1e-9);
            let held = scale > 0.0 && scale <= slack * p.radius / 20.0;
            let topped =
                scale <= slack * p.cloud_top && p.cloud_top <= scale / p.record.mix.kappa();
            assert!(held && topped && 0.0 < p.cloud_base, "{p:?}");
            assert!(p.cloud_base <= 0.5 * p.cloud_top, "{p:?}");
            assert!(p.cloud_base < p.cloud_top && p.cloud_top < p.atmosphere_height);
            assert_eq!(p.name, names::planet(&star.name, p.id.index));
            assert!(p.host.is_none() && p.moons <= 6 && !p.id.is_moon());
        }
    }
}

/// About 60 percent of the cells hold a galaxy, each with a radius, centre and arms inside its range.
#[test]
fn galaxies_stay_in_range() {
    let u = Universe::new(21);
    let galaxies = u.galaxies_near(GalaxyId(IVec3::new(40, -7, 3)), 6);
    let share = galaxies.len() as f64 / 13f64.powi(3);
    assert!((0.55..0.65).contains(&share), "{share}");
    let mut kinds = [0; 3];
    for g in &galaxies {
        assert!(within(g.radius / GLY, RADIUS_GLY), "{g:?}");
        assert!(
            g.center
                .to_array()
                .iter()
                .all(|&c| within(c / GALAXY_CELL, CENTER_SPAN))
        );
        let spiral = g.kind == GalaxyKind::Spiral;
        assert!(if spiral {
            (ARMS.0..=ARMS.1).contains(&g.arms)
        } else {
            g.arms == 0
        });
        assert!(!g.name.is_empty());
        kinds[g.kind as usize] += 1;
    }
    assert!(
        kinds[0] > kinds[1] && kinds[1] > kinds[2] && kinds[2] > 0,
        "{kinds:?}"
    );
}

/// Hot young stars form only in arms, the core and halo hold old stars, and red stars lead everywhere.
#[test]
fn regions_bias_the_star_class() {
    let count = |region: Region| {
        let mut classes = [0u32; 7];
        for seed in 0..20_000 {
            classes[star::build(StarId::default(), hash::mix(seed), region).class as usize] += 1;
        }
        classes
    };
    let (arm, disc, core) = (count(Region::Arm), count(Region::Disc), count(Region::Core));
    assert!(arm[0] > 0 && arm[1] > arm[0], "{arm:?}");
    assert_eq!((disc[0], disc[1]), (0, 0));
    assert_eq!(core[..3], [0, 0, 0]);
    assert_eq!(core, count(Region::Halo));
    for classes in [arm, disc, core] {
        assert!(
            classes[6] > 10_000 && classes[6] > 2 * classes[5],
            "{classes:?}"
        );
        assert!(
            classes[5] > classes[4] && classes[4] > classes[3],
            "{classes:?}"
        );
    }
}

/// Most stars carry a catalogue code, a few a proper name, and planets take the name of their star.
#[test]
fn names_are_mostly_catalogue_codes() {
    let names: Vec<String> = (0..5000).map(|seed| names::star(hash::mix(seed))).collect();
    let coded = names.iter().filter(|n| n.contains('-')).count();
    let plain = names.iter().filter(|n| !n.contains(['-', ' '])).count();
    assert!((2900..3500).contains(&coded), "{coded}");
    assert!((300..700).contains(&plain), "{plain}");
    assert!(
        names
            .iter()
            .all(|n| n.starts_with(|c: char| c.is_ascii_uppercase()))
    );
    assert_eq!(names::planet("A-21", 0), "A-21 b");
    assert_eq!(names::planet("Tavor 7", 7), "Tavor 7 i");
    println!("{:?}", &names[..24]);
}
