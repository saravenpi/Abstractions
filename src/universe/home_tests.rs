//! Home name tests: Kodya and Vereya are fixed for every seed and no other object carries them.

use super::*;

/// Returns true when a name holds one of the two home names as a word.
fn reserved(name: &str) -> bool {
    name.split(' ')
        .any(|word| word == HOME_PLANET_NAME || word == HOME_STAR_NAME)
}

/// The home planet is Kodya, the third planet of the yellow star Vereya, for every seed.
#[test]
fn home_names_are_fixed() {
    for seed in 0..200 {
        let u = Universe::new(seed);
        let home = u.home();
        let system = u.system(home.star);
        assert_eq!(system.star.name, "Vereya");
        assert_eq!(system.star.class, StarClass::G);
        assert_eq!(home.index, 2);
        assert_eq!(u.planet(home).name, "Kodya");
        let named = |name: &str| system.planets.iter().filter(|p| p.name == name).count();
        assert_eq!((named("Kodya"), named("Vereya")), (1, 0));
    }
}

/// No generator emits a home name: not for a star, a galaxy or a planet of another star.
#[test]
fn home_names_are_unique() {
    for seed in 0..200_000 {
        let seed = hash::mix(seed);
        assert!(!reserved(&names::star(seed)));
        assert!(!reserved(&names::galaxy(seed, seed.is_multiple_of(2))));
    }
    for seed in 0..8 {
        let u = Universe::new(seed);
        let home = u.home();
        let at = u.star(home.star).expect("the home star exists").pos;
        for star in u.stars_near(home.star.galaxy, at, 60.0) {
            assert_eq!(star.name == "Vereya", star.id == home.star);
            for planet in u.system(star.id).planets {
                assert_eq!(planet.name == "Kodya", planet.id == home);
                assert!(planet.id.star == home.star || !reserved(&planet.name));
            }
        }
        let galaxies = u.galaxies_near(home.star.galaxy, 2);
        assert!(galaxies.iter().all(|g| !reserved(&g.name)));
    }
}
