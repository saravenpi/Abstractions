//! Tests of how the climate shapes life: thick in the heart of a biome, thinner towards its edges,
//! nothing above the snow line, and the air villages and camps keep to.

use std::sync::Arc;

use bevy::math::Vec2;

use super::thrive::{DUNE_SEA, VILLAGE_AIR, climate_at, dune_sea, frozen, livable, vigor};
use crate::climate::{Climate, SNOW_LINE as FROST};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::Terrain;
use crate::world::biome::Biome;
use crate::world::globe::{Face, Globe};

/// Returns air of `temperature` kelvin and `moisture`, with no wind.
fn air(temperature: f32, moisture: f32) -> Climate {
    Climate {
        temperature,
        moisture,
        wind: Vec2::ZERO,
    }
}

/// Returns the vigor of `biome` at every kelvin from `from` towards `to`, in steps of a quarter.
fn sweep(biome: Biome, from: f32, to: f32, moisture: f32) -> Vec<f32> {
    let steps = ((to - from).abs() * 4.0) as usize;
    let step = (to - from).signum() * 0.25;
    (0..=steps)
        .map(|i| vigor(biome, &air(from + step * i as f32, moisture)))
        .collect()
}

/// Every biome is densest in its heart and thins out without a jump on both sides of its warmth
/// and as the air dries: no step of a quarter kelvin or of 0.004 of wetness takes off more than a
/// twentieth, and the far edge still keeps about a tenth of the plants.
#[test]
fn plants_thin_out_gradually_towards_the_edges() {
    for biome in Biome::ALL {
        let heart = (250..330)
            .map(|t| t as f32)
            .max_by(|a, b| vigor(biome, &air(*a, 2.0)).total_cmp(&vigor(biome, &air(*b, 2.0))))
            .unwrap();
        assert!(
            (vigor(biome, &air(heart, 2.0)) - 1.0).abs() < 1e-3,
            "{biome:?}"
        );
        for to in [heart - 30.0, heart + 30.0] {
            let line = sweep(biome, heart, to, 2.0);
            let pairs = line.windows(2);
            assert!(
                pairs.clone().all(|w| w[1] <= w[0] && w[0] - w[1] < 0.05),
                "{biome:?}"
            );
            assert!(
                line.last().is_some_and(|v| (0.089..0.5).contains(v)),
                "{biome:?}"
            );
        }
        let wet = |i: usize| vigor(biome, &air(heart, 1.0 - 0.004 * i as f32));
        let dry: Vec<f32> = (0..=250).map(wet).collect();
        assert!(
            dry.windows(2).all(|w| w[1] <= w[0] && w[0] - w[1] < 0.05),
            "{biome:?}"
        );
        assert!(dry[250] >= 0.089, "{biome:?} keeps {}", dry[250]);
    }
}

/// A forest is thick in its wet heart and thins towards the drier air of a steppe; a desert
/// keeps its sparse plants in air far drier than a forest needs.
#[test]
fn a_forest_thins_towards_the_steppe() {
    let forest = |moisture| vigor(Biome::Forest, &air(287.0, moisture));
    assert!(forest(1.0) > forest(0.5) && forest(0.5) > forest(0.25));
    assert!(forest(1.0) > 1.6 * forest(0.2));
    assert!(vigor(Biome::Desert, &air(305.0, 0.1)) > 2.0 * forest(0.1));
}

/// The snow line, the village range and the desert heart sit exactly at their numbers.
#[test]
fn the_limits_sit_at_their_numbers() {
    assert!(frozen(&air(FROST - 0.01, 1.0)) && !frozen(&air(FROST, 1.0)));
    let (cold, hot) = VILLAGE_AIR;
    assert!(livable(&air(cold, 0.5)) && livable(&air(hot, 0.5)));
    assert!(!livable(&air(cold - 0.01, 0.5)) && !livable(&air(hot + 0.01, 0.5)));
    assert!(DUNE_SEA.0 > VILLAGE_AIR.0 && DUNE_SEA.1 < 0.5);
}

/// Returns the face view of a 6 km globe of seed 5 with the home record changed into `class`,
/// `ocean` and `temperature`.
fn view(class: PlanetClass, ocean: f64, temperature: f64) -> Terrain {
    let record = PlanetRecord {
        class,
        ocean,
        temperature,
        ..PlanetRecord::TEMPERATE
    };
    let globe = Arc::new(Globe::new(5, 6000.0, 22.0).with_record(record));
    Terrain::on(globe.clone(), globe.face_chart(Face::PosZ))
}

/// The legacy plane has no climate, a globe has one that the same place always gives, and only a
/// hot dry desert has a heart nobody camps in.
#[test]
fn the_climate_is_read_on_globes_only() {
    let w = Vec2::new(120.0, -40.0);
    assert!(climate_at(&Terrain::new(5), w).is_none() && !dune_sea(&Terrain::new(5), w));
    let home = view(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN, 288.0);
    assert!(climate_at(&home, w).is_some_and(|air| air.temperature > 200.0));
    assert_eq!(climate_at(&home, w), climate_at(&home, w));
    assert!(!dune_sea(&home, w));
    let sands = view(PlanetClass::Desert, 0.0, 320.0);
    let hearts = (0..40).filter(|i| dune_sea(&sands, w + Vec2::X * 90.0 * *i as f32));
    assert!(hearts.count() > 0, "a hot dry desert has no heart");
}
