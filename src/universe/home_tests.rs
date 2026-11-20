//! Home tests: the home planet and its star carry generated names, and keep their star
//! temperature and their record for every seed; the home system is laid out around the orbit
//! that earns that record.

use std::collections::HashSet;

use super::*;

/// The home planet is the third planet of a yellow star, named from its star like every other
/// planet, and the only planet of its system with that name, for every seed.
#[test]
fn home_is_named_like_every_planet() {
    for seed in 0..200 {
        let u = Universe::new(seed);
        let home = u.home();
        let system = u.system(home.star);
        assert_eq!(system.star.class, StarClass::G);
        assert_eq!(home.index, 2);
        let name = u.planet(home).name;
        assert_eq!(name, names::planet(&system.star.name, home.index));
        assert_eq!(system.planets.iter().filter(|p| p.name == name).count(), 1);
    }
}

/// The home star and the home galaxy get a new name for each seed and the same name for the
/// same seed, so every player of one world reads the same names. The names of seeds 0 to 5
/// are printed.
#[test]
fn home_names_follow_the_seed() {
    let named = |seed| {
        let u = Universe::new(seed);
        let galaxy = u
            .galaxy(u.home().star.galaxy)
            .expect("the home cell holds a galaxy");
        (
            u.planet(u.home()).name,
            u.system(u.home().star).star.name,
            galaxy.name,
        )
    };
    let all: Vec<_> = (0..200).map(named).collect();
    assert!(
        all.iter()
            .zip(0..)
            .all(|(names, seed)| *names == named(seed))
    );
    let stars: HashSet<&String> = all.iter().map(|(_, star, _)| star).collect();
    let galaxies: HashSet<&String> = all.iter().map(|(_, _, galaxy)| galaxy).collect();
    assert!(
        stars.len() > 150 && galaxies.len() > 150,
        "{} {}",
        stars.len(),
        galaxies.len()
    );
    for (seed, (planet, star, galaxy)) in all.iter().take(6).enumerate() {
        println!("seed {seed}: {planet}, {star}, in {galaxy}");
    }
}

/// The home star is at 5,780 K and the home planet keeps the temperate record, for every seed.
///
/// The classes of the home system are printed for seeds 0 to 20, the home planet in capitals.
#[test]
fn home_record_is_temperate() {
    for seed in 0..200 {
        let u = Universe::new(seed);
        let system = u.system(u.home().star);
        assert_eq!(system.star.temperature, 5780.0);
        assert_eq!(u.planet(u.home()).record, PlanetRecord::TEMPERATE);
        let label = |p: &PlanetSpec| match p.id == u.home() {
            true => p.record.class.label().to_uppercase(),
            false => p.record.class.label().to_string(),
        };
        let classes: Vec<String> = system.planets.iter().map(label).collect();
        if seed <= 20 {
            println!("seed {seed}: {}", classes.join(", "));
        }
    }
}

/// The home planet sits where its star gives it the flux of its record, so the pinned record is
/// the one its orbit earns: flux 1, in the habitable zone. It is the third of at least 5
/// planets on orbits that grow outward, the two inside it get more light and the two or more
/// beyond it get less.
#[test]
fn home_orbit_earns_its_record() {
    for seed in 0..200 {
        let u = Universe::new(seed);
        let system = u.system(u.home().star);
        let (star, planets) = (&system.star, &system.planets);
        let home = &planets[usize::from(u.home().index)];
        assert_eq!(home.orbit_radius, zone::FLUX_ORBIT);
        assert_eq!(
            zone::flux(star.luminosity, home.orbit_radius),
            home.record.flux
        );
        assert_eq!(
            zone::zone(star.temperature, home.record.flux),
            home.record.zone
        );
        assert!(
            (5..=PLANETS.1).contains(&(planets.len() as u8)),
            "seed {seed}"
        );
        let grows = |pair: &[PlanetSpec]| pair[1].orbit_radius > 1.04 * pair[0].orbit_radius;
        assert!(planets.windows(2).all(grows), "seed {seed}");
        let (inner, outer) = planets.split_at(usize::from(u.home().index));
        assert!(inner.iter().all(|p| p.record.flux > 1.5), "seed {seed}");
        assert!(
            outer[1..].iter().all(|p| p.record.flux < 0.95),
            "seed {seed}"
        );
        assert!(planets.iter().all(|p| p.orbit_radius < ORBIT.1));
    }
}

/// The home planet has no moons and no rings, and the outermost planet of the home system is a gas giant
/// with at least two moons, all of them solid ground to land on, for every seed. The home
/// systems of seeds 0 to 20 are printed: class, moons and rings of each planet.
#[test]
fn home_has_a_giant_with_moons() {
    for seed in 0..200 {
        let u = Universe::new(seed);
        let system = u.system(u.home().star);
        let homeworld = u.planet(u.home());
        assert_eq!((homeworld.moons, homeworld.rings), (0, None), "seed {seed}");
        assert!(u.moons(u.home()).is_empty());
        let outer = system.planets.last().expect("the home system has planets");
        assert_eq!(outer.record.class, PlanetClass::GasGiant, "seed {seed}");
        assert_eq!(outer.record.zone, Zone::Cold, "seed {seed}");
        let moons = moon::of(&system.star, outer);
        assert!(moons.len() >= 2, "seed {seed}");
        let solid = |m: &PlanetSpec| m.record.class != PlanetClass::GasGiant;
        assert!(
            moons
                .iter()
                .all(|m| solid(m) && m.radius <= PLANET_RADIUS.1)
        );
        let entry = |p: &PlanetSpec| {
            let ring = if p.rings.is_some() { " ringed" } else { "" };
            format!("{} {}m{ring}", p.record.class.label(), p.moons)
        };
        let entries: Vec<String> = system.planets.iter().map(entry).collect();
        if seed <= 20 {
            println!("seed {seed}: {}", entries.join(", "));
        }
    }
}
