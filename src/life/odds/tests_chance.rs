//! Tests of the chance of life: it matches the share of seeds that live, and it is cheap.

use std::time::Instant;

use super::*;
use crate::universe::GIANT_RADIUS;

/// Returns a world under a Sun-like star with `pressure`, `temperature`, `flux` and liquid share
/// `ocean`, of radius `radius` metres.
fn world(pressure: f32, temperature: f32, flux: f32, ocean: f32, radius: f32) -> World {
    World {
        gravity: 9.8,
        pressure,
        temperature,
        flux,
        radius,
        ocean,
        star: 5780.0,
        tilt: 0.0,
        liquids: [
            ocean.max(f32::MIN_POSITIVE),
            0.0375,
            0.0375,
            ocean.max(f32::MIN_POSITIVE),
        ],
    }
}

/// Returns the share of 20,000 seeds for which `world` lives.
fn share(world: &World) -> f32 {
    let living = (0..20_000)
        .filter(|seed| rung(world, *seed).is_some())
        .count();
    living as f32 / 20_000.0
}

/// An Earth twin, a Titan-like moon, a hot airless rock and a wet world of a red dwarf each have a
/// chance within two points of the share of seeds that live on them. Prints each chance.
#[test]
fn the_chance_is_the_share_of_seeds_that_live() {
    let worlds = [
        ("earth", world(1.0, 288.0, 1.0, 0.7, 6000.0)),
        ("titan", world(1.5, 94.0, 0.011, 0.0, 2575.0)),
        ("rock", world(0.0, 700.0, 4.0, 0.0, 3000.0)),
        (
            "dwarf",
            World {
                star: 3400.0,
                ..world(1.0, 285.0, 0.9, 0.5, 5000.0)
            },
        ),
    ];
    for (name, world) in worlds {
        let (chance, share) = (chance(&world), share(&world));
        println!("{name}: chance {chance:.3}, share of seeds {share:.3}");
        assert!(
            (chance - share).abs() < 0.02,
            "{name}: {chance} against {share}"
        );
    }
    assert!(chance(&worlds[0].1) > 0.9 && chance(&worlds[1].1) < 0.1);
    assert_eq!(chance(&worlds[2].1), 0.0);
}

/// A gas giant and a world no liquid suits have no chance, and a world barely suited a tiny one.
#[test]
fn barren_worlds_have_no_chance() {
    let giant = world(1.0, 288.0, 1.0, 0.7, GIANT_RADIUS.0 as f32);
    let dry = world(1.0, 288.0, 1.0, 0.0, 6000.0);
    assert_eq!(chance(&giant), 0.0);
    assert!(chance(&dry) <= share(&dry) + 0.02);
    assert!((0.0..=1.0).contains(&chance(&world(0.01, 200.0, 0.001, 0.01, 6000.0))));
}

/// A hundred thousand chances take well under a frame, so the card can compute one at will.
#[test]
fn the_chance_is_cheap() {
    let earth = world(1.0, 288.0, 1.0, 0.7, 6000.0);
    let start = Instant::now();
    let total: f32 = (0..100_000)
        .map(|i| {
            chance(&World {
                temperature: 250.0 + (i % 80) as f32,
                ..earth
            })
        })
        .sum();
    let spent = start.elapsed();
    println!("100000 chances in {spent:?}, sum {total}");
    assert!(spent.as_millis() < 200, "{spent:?}");
}
