//! Tests of the population field.

use super::{Site, density, herds};
use crate::life::herd::fixtures::{meadow_life, site};

/// Producers have no density, apex hunters are the rarest animals, and a meadow holds a few herds per km².
#[test]
fn apex_is_rarest_and_meadow_is_sane() {
    let life = meadow_life();
    let meadow = site(50.0, 1.0);
    assert_eq!(density(&life.species[0], &meadow), 0.0);
    let apex = density(&life.species[5], &meadow);
    assert!(apex > 0.0);
    for species in &life.species[1..5] {
        assert!(
            density(species, &meadow) > apex,
            "{} vs apex {apex}",
            species.name
        );
    }
    let total: f32 = life.species.iter().map(|s| herds(s, &meadow)).sum();
    assert!((1.0..10.0).contains(&total), "{total} herds per km²");
}

/// Habitat decides where a species can live at all.
#[test]
fn habitat_matches_the_site() {
    let life = meadow_life();
    let (land, sea, beach) = (site(50.0, 1.0), site(-20.0, 1.0), site(2.0, 1.0));
    let (ground, water, shore) = (&life.species[1], &life.species[6], &life.species[7]);
    assert!(density(ground, &land) > 0.0 && density(ground, &sea) == 0.0);
    assert!(density(water, &sea) > 0.0 && density(water, &land) == 0.0);
    assert!(density(shore, &beach) > 0.0 && density(shore, &land) == 0.0);
}

/// Density falls smoothly to nothing outside the tolerated climate.
#[test]
fn tolerance_is_smooth() {
    let life = meadow_life();
    let grazer = &life.species[1];
    let at = |temperature| {
        density(
            grazer,
            &Site {
                temperature,
                ..site(50.0, 1.0)
            },
        )
    };
    assert!(at(288.0) > at(318.0) && at(318.0) > 0.0);
    assert_eq!(at(330.0), 0.0);
    assert_eq!(at(240.0), 0.0);
}
