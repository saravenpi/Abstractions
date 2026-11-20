//! Tests of herd paths on steep ground: the walk of a herd centre never climbs ground steeper than
//! its body holds, on a planet of ridges steep enough to stop it.

use std::collections::HashMap;

use bevy::math::DVec3;

use super::fixtures::{light, meadow_life, site};
use super::path::walkable;
use super::tests::{RADIUS, land};
use super::{Ground, Search, near};
use crate::life::body::slope;

/// Returns the ground of a planet of ridges about 470 m apart whose sides reach 58 degrees.
fn ridges(dir: DVec3) -> f32 {
    let d = dir.as_vec3();
    150.0 + 120.0 * (80.0 * d.x).sin() * (0.5 + 0.5 * (7.0 * d.z).cos())
}

/// A path up a cliff or across the sea is not walkable for a walker, one along flat ground is, and a
/// flyer takes any path.
#[test]
fn walkable_paths_follow_the_slope() {
    let a = land();
    let b = (a + DVec3::new(0.0, 0.0, 0.05)).normalize();
    let wall = |dir: DVec3| if dir.z > (a.z + b.z) * 0.5 { 50.0 } else { 0.0 };
    let land = |slope: f32| (slope, 0.0);
    assert!(!walkable(&wall, (a, b), RADIUS, land(0.7)));
    assert!(walkable(&|_| 10.0, (a, b), RADIUS, land(0.7)));
    assert!(
        !walkable(&|_| -3.0, (a, b), RADIUS, land(0.7)),
        "a walker crossed the sea"
    );
    assert!(walkable(
        &wall,
        (a, b),
        RADIUS,
        (f32::INFINITY.atan(), f32::NEG_INFINITY)
    ));
}

/// Returns the steepest grade over its limit any herd of `life` walks on the ridges, read between
/// its centres two seconds apart over twenty minutes, and how many steps were read.
fn steepest(life: &crate::life::Life, ground: &Ground) -> (f64, usize) {
    let mut last: HashMap<u64, DVec3> = HashMap::new();
    let (mut steps, mut worst) = (0, 0.0_f64);
    for k in 0..600 {
        let search = Search {
            at: land(),
            radius: 1500.0,
            planet: RADIUS,
            time: 2.0 * f64::from(k),
        };
        for herd in near(life, ground, &search) {
            let limit = f64::from(slope(&life.species[usize::from(herd.species)].body).tan());
            let before = last.insert(herd.id, herd.centre).unwrap_or(herd.centre);
            let run = super::arc(before, herd.centre) * RADIUS;
            let rise = f64::from((ridges(herd.centre) - ridges(before)).abs());
            if run > 2.0 {
                worst = worst.max(rise / run / limit);
                steps += 1;
            }
        }
    }
    (worst, steps)
}

/// On the ridges, every herd centre moves over ground no steeper than its species holds, read over
/// steps of a few metres.
#[test]
fn herds_walk_around_ridges() {
    let life = meadow_life();
    let ground = Ground {
        sample: &|dir| site(ridges(dir), 1.0),
        height: &ridges,
        light: &light,
    };
    let (worst, steps) = steepest(&life, &ground);
    println!("{steps} steps, worst grade {worst:.2} of the limit");
    assert!(steps > 100, "only {steps} steps");
    assert!(worst <= 1.3, "a herd climbed {worst:.2} times its slope");
}
