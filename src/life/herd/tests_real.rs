//! Tests of the herds of real planets: each comes from the planet's own web and climate.

use bevy::math::DVec3;

use super::fixtures::{height, light};
use super::tests::{RADIUS, land};
use super::{Ground, Herd, Search, near};
use crate::life::census::Site;
use crate::life::{Guild, Life, World};
use crate::world::Biome;

/// Returns the herds of `life` within 2.5 km of [`land`] at 600 s, on hills whose air is `heat`
/// kelvin hotter than `world`.
fn look(life: &Life, world: &World, heat: f32) -> Vec<Herd> {
    let sample = |dir: DVec3| Site {
        height: height(dir),
        temperature: world.temperature + heat,
        moisture: 0.6,
        biome: Biome::Meadow,
        liquid: height(dir) < 0.0,
        daylight: 1.0,
    };
    let ground = Ground {
        sample: &sample,
        light: &light,
    };
    let search = Search {
        at: land(),
        radius: 2500.0,
        planet: RADIUS,
        time: 600.0,
    };
    near(life, &ground, &search)
}

/// Each living planet gets herds of animals on ground at its own temperature, and none where the air
/// is 80 K hotter than it is.
#[test]
fn real_lives_have_herds_from_their_climate() {
    let mut herded = 0;
    for (star, planet, life) in crate::life::web::tests::lives(40) {
        let world = World::of(&planet, &star);
        let herds = look(&life, &world, 0.0);
        let guild = |h: &Herd| life.species[usize::from(h.species)].role.guild;
        assert!(herds.iter().all(|h| guild(h) != Guild::Producer));
        assert!(look(&life, &world, 80.0).is_empty());
        herded += usize::from(!herds.is_empty());
    }
    assert!(herded > 0, "no living planet has herds");
}
