//! Tests of the herds marked for the compass: the nearest herds of animals big enough to see
//! within 300 m of the robot are marked, nearest first, and none farther.

use super::herds::{Herds, herd_arc};
use super::near::Nearby;
use super::rig::app;
use super::stage::{robot, run};
use super::tune::{MARK_REACH, MARK_SIZE, MARKS};
use super::want::drawn;
use crate::life::Species;
use crate::life::body::Locomotion;
use crate::world::Terrain;

/// Returns true when herds of `species` are worth a mark.
fn marked(species: &Species) -> bool {
    drawn(species)
        && species.body.locomotion != Locomotion::Swarm
        && species.body.length >= MARK_SIZE
}

/// With the robot in a herd, the herds within 300 m worth a mark are marked, nearest first, and
/// none farther.
#[test]
fn herds_within_reach_are_marked() {
    let (mut app, _) = app(true);
    run(&mut app, 2.0, 0.1);
    let me = robot(&mut app).normalize();
    let world = app.world();
    let globe = world.resource::<Terrain>().globe().clone();
    let life = globe.life().expect("a living planet");
    let within: Vec<f64> = world
        .resource::<Herds>()
        .herds
        .iter()
        .filter(|h| marked(&life.species[usize::from(h.species)]))
        .map(|h| herd_arc(h.centre, me) * globe.radius())
        .filter(|m| *m <= MARK_REACH)
        .collect();
    let nearby = world.resource::<Nearby>();
    let metres: Vec<f64> = nearby.herds.iter().map(|s| s.metres).collect();
    println!("marked herds at {metres:.0?} m of {within:.0?} m");
    assert!(nearby.body.is_some());
    assert!(within.is_empty() || !metres.is_empty());
    assert!(metres.len() <= within.len().min(MARKS));
    assert!(metres.windows(2).all(|w| w[0] <= w[1]));
    assert!(metres.iter().all(|m| *m <= MARK_REACH));
    let nearest = within.iter().copied().fold(f64::MAX, f64::min);
    assert!(metres.first().is_none_or(|m| (m - nearest).abs() < 1.0));
}
