//! Tests of life histories.

use super::of;
use crate::life::herd::fixtures::{homeworld, role};
use crate::life::{Activity, Guild, Habitat, Rung, World};

/// The same seed gives the same history.
#[test]
fn same_seed_same_history() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let a = of(7, &grazer, 100.0, Rung::Skeletal, &homeworld());
    assert_eq!(a, of(7, &grazer, 100.0, Rung::Skeletal, &homeworld()));
}

/// Large species live longer, have fewer young, care more and range wider; a 100 kg grazer ranges
/// about 400 m in herds of 5 to 30.
#[test]
fn mass_sets_the_pace_of_life() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let small = of(1, &grazer, 0.01, Rung::Skeletal, &homeworld());
    let mid = of(1, &grazer, 100.0, Rung::Skeletal, &homeworld());
    let giant = of(1, &grazer, 5000.0, Rung::Skeletal, &homeworld());
    assert!(small.lifespan < mid.lifespan && mid.lifespan < giant.lifespan);
    assert!(
        small.clutch > 10 && giant.clutch == 1,
        "{} {}",
        small.clutch,
        giant.clutch
    );
    assert!(small.care < mid.care && mid.care < giant.care);
    assert!((mid.range - 400.0).abs() < 50.0, "{}", mid.range);
    assert!(giant.range > mid.range && mid.range > small.range);
    assert_eq!((small.group_min, small.group_max), (20, 60));
    assert_eq!((mid.group_min, mid.group_max), (5, 30));
}

/// Groups follow the guild: lone ambushers, hunting packs only on old worlds.
#[test]
fn guild_sets_the_group() {
    let hunter = role(Guild::Hunter, Habitat::Ground, Activity::Night, 3.0);
    let ambusher = role(Guild::Ambusher, Habitat::Ground, Activity::Night, 3.0);
    let browser = role(Guild::Browser, Habitat::Ground, Activity::Day, 2.0);
    let group = |r, rung| {
        let h = of(3, &r, 80.0, rung, &homeworld());
        (h.group_min, h.group_max)
    };
    assert_eq!(group(hunter, Rung::Skeletal), (1, 4));
    assert!(group(hunter, Rung::Old).0 > 1);
    assert_eq!(group(ambusher, Rung::Old), (1, 1));
    assert_eq!(group(browser, Rung::Skeletal), (2, 8));
}

/// Speed peaks at middle sizes and falls on heavy worlds; hunters range wider than grazers.
#[test]
fn speed_and_range_follow_body_and_world() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let hunter = role(Guild::Hunter, Habitat::Ground, Activity::Day, 3.0);
    let light = World {
        gravity: 9.81,
        ..homeworld()
    };
    let speed = |mass, world: &World| of(2, &grazer, mass, Rung::Skeletal, world).speed;
    assert!(speed(0.01, &light) < speed(100.0, &light));
    assert!(speed(20000.0, &light) < speed(100.0, &light));
    assert!(speed(100.0, &homeworld()) < speed(100.0, &light) * 0.75);
    let range = |r, mass| of(2, &r, mass, Rung::Skeletal, &homeworld()).range;
    assert!(range(hunter, 100.0) > 2.0 * range(grazer, 100.0));
}
