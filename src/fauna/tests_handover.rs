//! Tests of the hand-over between the herds simulated far from the player and the animals drawn near:
//! an animal is drawn where the simulation puts it, never jumps from one frame to the next, and the herd
//! marks stand among the drawn animals of their herd.

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Nearby;
use super::herds::{Herds, member};
use super::rig::app;
use super::stage::{run, staying};
use super::want::Lead;
use crate::space::Clock;
use crate::world::Terrain;

/// Seconds per frame of the hand-over tests.
const DT: f32 = 1.0 / 30.0;

/// Metres from the middle of its herd's drawn animals within which a mark must stand.
const MARK_NEAR: f64 = 3.0;

/// Returns the planet position of every staying animal of `app`, by the herd and slot that lead it.
fn places(app: &mut App) -> HashMap<(u64, u8), DVec3> {
    staying(app)
        .iter()
        .map(|b| (b.lead.key(), b.spot.planet(b.motion.root.place)))
        .collect()
}

/// Every herd member just drawn stands where the simulation puts its member, within half a metre: the
/// drawn animal takes over from the simulated one where it is.
#[test]
fn animals_are_drawn_where_the_herd_is() {
    let (mut app, _) = app(true);
    run(&mut app, DT, DT);
    let world = app.world();
    let (time, radius) = (
        world.resource::<Clock>().seconds,
        world.resource::<Terrain>().globe().radius(),
    );
    let herds = world.resource::<Herds>().herds.clone();
    let mut checked = 0;
    for ((id, slot), at) in places(&mut app) {
        let Some(herd) = herds.iter().find(|h| h.id == id) else {
            continue;
        };
        let Some((_, dir)) = member(herd, slot, time, radius) else {
            continue;
        };
        let gap = (at.normalize() - dir).length() * radius;
        assert!(
            gap < 0.5,
            "herd {id} member {slot} drawn {gap} m from its place"
        );
        checked += 1;
    }
    assert!(checked > 0, "no herd member drawn");
}

/// Over ten seconds of herds grazing, fleeing and moving to new frames, no drawn animal moves farther
/// in one frame than twice its top speed allows, plus 10 cm: no animal jumps.
#[test]
fn no_drawn_animal_jumps() {
    let (mut app, _) = app(true);
    run(&mut app, 1.0, DT);
    let mut last = places(&mut app);
    for frame in 0..300 {
        run(&mut app, DT, DT);
        let now = places(&mut app);
        for (key, at) in &now {
            let Some(before) = last.get(key) else {
                continue;
            };
            let step = at.distance(*before);
            assert!(
                step < 2.0 * 30.0 * f64::from(DT) + 0.1,
                "frame {frame}: {key:?} jumped {step} m"
            );
        }
        last = now;
    }
}

/// After a minute among the herds, every herd mark stands within [`MARK_NEAR`] of the middle of the drawn
/// animals of its herd, while some are drawn.
#[test]
fn marks_stand_among_the_drawn_animals() {
    let (mut app, _) = app(true);
    run(&mut app, 60.0, 0.1);
    let places = places(&mut app);
    let marks = app.world().resource::<Nearby>().herds.clone();
    for mark in &marks {
        let mine: Vec<DVec3> = places
            .iter()
            .filter(|(k, _)| k.0 == mark.id)
            .map(|(_, at)| *at)
            .collect();
        if mine.is_empty() {
            continue;
        }
        let middle = mine.iter().sum::<DVec3>() / mine.len() as f64;
        let gap = mark.at.distance(middle);
        assert!(
            gap < MARK_NEAR,
            "the mark of herd {} is {gap} m from its animals",
            mark.id
        );
    }
    println!("{} marks checked", marks.len());
    let leads = staying(&mut app)
        .iter()
        .filter(|b| matches!(b.lead, Lead::Herd { .. }))
        .count();
    assert!(leads > 0, "no herd member drawn");
}
