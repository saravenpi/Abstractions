//! Tests of how the climate shapes life: thick in the heart of a biome, thinner towards its edges,
//! nothing above the snow line, and a climate on globes only.

use std::sync::Arc;

use bevy::math::Vec2;

use super::thrive::{TREE_LINE, climate_at, frozen, vigor};
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

/// The snow line and the tree line of the taiga sit exactly at their numbers.
#[test]
fn the_limits_sit_at_their_numbers() {
    let meadow = |t: f32| frozen(Biome::Meadow, &air(t, 1.0));
    assert!(meadow(FROST - 0.01) && !meadow(FROST));
    let taiga = |t: f32| frozen(Biome::Taiga, &air(t, 1.0));
    assert!(!taiga(FROST - 10.0) && taiga(TREE_LINE - 0.01) && !taiga(TREE_LINE));
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

/// The legacy plane has no climate, and a globe has one that the same place always gives.
#[test]
fn the_climate_is_read_on_globes_only() {
    let w = Vec2::new(120.0, -40.0);
    assert!(climate_at(&Terrain::new(5), w).is_none());
    let home = view(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN, 288.0);
    assert!(climate_at(&home, w).is_some_and(|air| air.temperature > 200.0));
    assert_eq!(climate_at(&home, w), climate_at(&home, w));
}
