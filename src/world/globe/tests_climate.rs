//! Tests of the climate on the globe: biomes follow the climate field, the season moves nothing on the
//! ground, and the same seed always gives the same ground whatever order and thread it is read in.

use bevy::math::DVec3;

use super::Globe;
use super::season::MEAN_SEASON;
use super::tests_landform::world;
use super::tests_record::sphere_dirs;
use crate::climate;
use crate::universe::PlanetClass;
use crate::world::{Biome, SEA_LEVEL, Surface};

/// On the sphere the biome of a world of islands is the one its climate gives: a scorched dry world is
/// desert from pole to pole, a frozen one taiga.
#[test]
fn sphere_biomes_follow_the_climate() {
    let (hot, cold) = (
        world(PlanetClass::Island, 0.1, 345.0),
        world(PlanetClass::Island, 0.6, 215.0),
    );
    for dir in sphere_dirs(1500) {
        assert_eq!(hot.biome(dir), Biome::Desert, "{dir}");
        assert_eq!(cold.biome(dir), Biome::Taiga, "{dir}");
    }
    let home = Globe::new(42, 6000.0, 22.0);
    let jitter = |d: DVec3| {
        let p = d * home.radius() / 90.0;
        crate::world::islands::gradient(crate::world::islands::sub_seed(42, 18), p.to_array())
            as f32
    };
    for dir in sphere_dirs(1500) {
        let climate = climate::at(&home.setting, &home.spot(dir), 0.25);
        assert_eq!(
            home.biome(dir),
            climate::biome(&climate, jitter(dir)),
            "{dir}"
        );
    }
}

/// The season starts at the annual mean and wraps into 0 to 1; setting it moves the climate of the moment
/// and nothing of the ground: heights, biomes and surfaces stay bit for bit.
#[test]
fn the_season_moves_only_the_climate() {
    let globe = Globe::new(5, 6000.0, 22.0);
    assert_eq!(globe.season(), MEAN_SEASON);
    let before: Vec<(u32, Biome, Surface)> = sphere_dirs(600)
        .map(|d| (globe.height(d).to_bits(), globe.biome(d), globe.surface(d)))
        .collect();
    globe.set_season(1.75);
    assert_eq!(globe.season(), 0.75);
    let after: Vec<(u32, Biome, Surface)> = sphere_dirs(600)
        .map(|d| (globe.height(d).to_bits(), globe.biome(d), globe.surface(d)))
        .collect();
    assert_eq!(before, after);
    let dir = DVec3::new(0.3, 0.5, -0.8).normalize();
    assert_eq!(globe.climate(dir), globe.climate_in(dir, 0.75));
}

/// Returns the heights, biomes and surfaces of `globe` at `dirs`, read in that order.
fn read(globe: &Globe, dirs: &[DVec3]) -> Vec<(u32, Biome, Surface)> {
    let at = |&d: &DVec3| (globe.height(d).to_bits(), globe.biome(d), globe.surface(d));
    dirs.iter().map(at).collect()
}

/// Two globes of one seed give the same ground bit for bit, one read forwards here and one read backwards
/// on a fresh thread, which holds none of the cells the first read, and so does the first globe read
/// backwards over its warm cells: nothing depends on what the memories of cells held.
#[test]
fn the_same_seed_gives_the_same_ground() {
    let dirs: Vec<DVec3> = sphere_dirs(1500).collect();
    let a = Globe::new(9, 6000.0, 22.0);
    let forwards = read(&a, &dirs);
    let turned: Vec<DVec3> = dirs.iter().rev().copied().collect();
    let mut warm = read(&a, &turned);
    warm.reverse();
    assert_eq!(forwards, warm);
    let other = std::thread::spawn(move || read(&Globe::new(9, 6000.0, 22.0), &turned));
    let mut backwards = other.join().expect("the thread reads the ground");
    backwards.reverse();
    assert_eq!(forwards, backwards);
    let land = forwards
        .iter()
        .filter(|(h, _, _)| f32::from_bits(*h) > SEA_LEVEL)
        .count();
    assert!(land > 300, "{land} land sites");
}

/// The land of the home planet holds all six biomes in its latitude bands, none of them over half of it.
#[test]
fn home_land_holds_every_biome() {
    let home = Globe::new(42, 6000.0, 22.0);
    let land: Vec<Biome> = sphere_dirs(8000)
        .filter(|&d| home.height(d) > SEA_LEVEL)
        .map(|d| home.biome(d))
        .collect();
    for biome in Biome::ALL {
        let share = land.iter().filter(|&&b| b == biome).count() as f32 / land.len() as f32;
        assert!(
            (0.02..0.5).contains(&share),
            "{biome:?} covers {share} of the land"
        );
    }
}
