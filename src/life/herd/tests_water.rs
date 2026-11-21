//! Tests of herds and water: far from the player, where no animal is drawn, the centre of a herd never
//! walks through a lake deeper than the legs of its species, on a planet strewn with small lakes.

use bevy::math::DVec3;

use super::fixtures::{light, meadow_life, site};
use super::tests::{RADIUS, land};
use super::{Ground, Search, floor, near};

/// Returns the ground of a planet of land strewn with lakes 25 to 35 m wide about 94 m apart.
fn lakes(dir: DVec3) -> f32 {
    let d = dir.as_vec3();
    3.0 - 8.0 * (400.0 * d.x).sin() * (400.0 * d.z).sin()
}

/// Over twenty minutes, read every two seconds, no herd centre of a species that wades no deeper than
/// its legs stands in a lake deeper than its legs.
#[test]
fn far_herds_keep_out_of_lakes() {
    let life = meadow_life();
    let ground = Ground {
        sample: &|dir| site(lakes(dir), 1.0),
        height: &lakes,
        light: &light,
    };
    let mut read = 0;
    for k in 0..600 {
        let search = Search {
            at: land(),
            radius: 1500.0,
            planet: RADIUS,
            time: 2.0 * f64::from(k),
        };
        for herd in near(&life, &ground, &search) {
            let species = &life.species[usize::from(herd.species)];
            let legs = species.body.height.max(0.1);
            let depth = -lakes(herd.centre);
            assert!(
                lakes(herd.centre) >= floor(species) - legs,
                "herd {} of species {} {depth:.2} m deep in a lake at {} s",
                herd.id,
                herd.species,
                search.time
            );
            read += 1;
        }
    }
    println!("{read} herd centres read, none in a lake");
    assert!(read > 100, "only {read} herd centres read");
}
