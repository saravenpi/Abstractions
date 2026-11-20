//! Tests of meeting big animals: on living home planets with skeletal life or older, herds of plant
//! eaters tall enough to hunt live within a walk of open land, by the general rules of ecology.

use std::sync::Arc;

use bevy::math::DVec3;

use super::herds::{Land, herd_arc, read};
use crate::home::Base;
use crate::life::body::Locomotion;
use crate::life::herd::{Herd, Search, near};
use crate::life::{self, Guild, Life, Rung, Species};
use crate::universe::hash::Rng;
use crate::universe::{BodyId, PlanetSpec, Universe};
use crate::world::Globe;

/// Hip height in metres from which an animal is big game.
const TALL: f32 = 0.8;

/// Metres within which a herd of big game is looked for around a point.
const WALK: f64 = 1500.0;

/// A home planet of a seed: its body, record, life and globe.
type Home = (BodyId, PlanetSpec, Life, Arc<Globe>);

/// Returns the home planet of universe `seed` when it holds skeletal life or older.
fn home(seed: u64) -> Option<Home> {
    let universe = Universe::new(seed);
    let body = universe.home();
    let spec = universe.planet(body);
    let life = life::of(&spec, &universe.system(body.star).star)?;
    let globe = Arc::new(Globe::of(&spec).with_life(Some(life.clone())));
    (life.rung >= Rung::Skeletal).then_some((body, spec, life, globe))
}

/// Returns true for a plant eater that walks on legs at least [`TALL`] at the hip.
fn game(s: &Species) -> bool {
    s.body.locomotion == Locomotion::Walk && s.body.height >= TALL && s.role.level < 2.3
}

/// Returns the metres from unit direction `at` to the nearest herd of big game within `radius` of
/// the home planet `home`, at universe time 600 s.
fn nearest(home: &Home, at: DVec3, radius: f64) -> Option<f64> {
    let (_, spec, life, globe) = home;
    let land = Land {
        globe,
        life,
        planet: spec,
    };
    let search = Search {
        at,
        radius,
        planet: globe.radius(),
        time: 600.0,
    };
    let herds: Vec<Herd> = read(land, 600.0, |g| near(life, g, &search));
    herds
        .iter()
        .filter(|h| game(&life.species[usize::from(h.species)]))
        .map(|h| herd_arc(h.centre, at) * globe.radius())
        .min_by(f64::total_cmp)
}

/// Returns `count` unit directions of dry open land of `globe`, drawn from `seed`.
fn open_land(globe: &Globe, seed: u64, count: usize) -> Vec<DVec3> {
    let mut rng = Rng::new(seed);
    let mut found = Vec::with_capacity(count);
    while found.len() < count {
        let mut axis = || rng.range(-1.0, 1.0);
        let dir = DVec3::new(axis(), axis(), axis()).normalize_or(DVec3::Y);
        if globe.height(dir) > 2.0 {
            found.push(dir);
        }
    }
    found
}

/// On the home planets of seeds 0 to 20 that hold skeletal life or older, three in five points of open
/// land or more have a herd of plant eaters at least 0.8 m at the hip within 1.5 km.
#[test]
fn big_game_lives_within_a_walk() {
    let (mut met, mut points) = (0, 0);
    for seed in 0..21 {
        let Some(home) = home(seed) else {
            continue;
        };
        for at in open_land(&home.3, seed, 3) {
            points += 1;
            met += usize::from(nearest(&home, at, WALK).is_some());
        }
    }
    println!("big game within {WALK} m of {met} of {points} points");
    assert!(points >= 15, "only {points} points");
    assert!(
        met * 5 >= points * 3,
        "big game near only {met} of {points} points"
    );
}

/// Prints each plant eater of `life`, the life of the home planet of universe `seed`.
fn show(seed: u64, life: &Life) {
    let eaters = life.species.iter();
    for s in eaters.filter(|s| s.role.level < 2.3 && s.role.guild != Guild::Producer) {
        println!(
            "seed {seed} {} {:?} {:?}/{} {:?} {:?} {:.1} kg hip {:.2} m group {}-{} range {:.0} m",
            s.name,
            s.role.guild,
            s.plan,
            s.archetype,
            s.body.locomotion,
            s.role.habitat,
            s.history.mass,
            s.body.height,
            s.history.group_min,
            s.history.group_max,
            s.history.range,
        );
    }
}

/// Prints, for home planets of seeds 0 to 20, each plant eater and the nearest herd of big game to
/// the wake point and to random land points.
#[test]
#[ignore]
fn survey() {
    for seed in 0..21 {
        let Some(home) = home(seed) else {
            println!("seed {seed}: no skeletal life");
            continue;
        };
        show(seed, &home.2);
        let wake = Base::on(&home.3, home.0).wake.normalize();
        let land: Vec<Option<f64>> = open_land(&home.3, seed, 6)
            .into_iter()
            .map(|at| nearest(&home, at, 3000.0).map(f64::round))
            .collect();
        println!(
            "seed {seed} wake {:?} land {land:?}",
            nearest(&home, wake, 3000.0).map(f64::round)
        );
    }
}
