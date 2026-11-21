//! Tests of animals that live while the robot watches, run through the fauna systems as the game runs
//! them: a grazing herd steps about as it feeds, and a shy animal runs from a robot that walks up to
//! it. The attacker that runs in is tested with the attacks (`attack::tests_close`).

use std::sync::Arc;

use bevy::math::{DVec3, Vec2, Vec3};
use bevy::prelude::*;

use super::herds::{Land, read};
use super::place::beside;
use super::rig::{SEED, START, app_of, planet};
use super::senses::stance::plant_eater;
use super::senses::tune::{PREY_SHARE, ROBOT_KG};
use super::stage::{put, robot, run, stage, staying};
use super::want::{Lead, drawn};
use crate::life::body::Locomotion;
use crate::life::herd::{Herd, HerdState, Search, near};
use crate::life::{Life, Species};
use crate::universe::{BodyId, Universe};
use crate::world::Globe;

/// Returns true for a drawn walker of at least 20 kg, whose grazing steps are long enough to see.
fn big_walker(species: &Species) -> bool {
    drawn(species) && species.body.locomotion == Locomotion::Walk && species.history.mass >= 20.0
}

/// Returns the spots of the planet the rig looks for herds around, as unit directions.
fn spots() -> impl Iterator<Item = DVec3> {
    (0..24).map(|i| {
        let a = f64::from(i) * 0.7;
        DVec3::new(a.cos(), 0.3 * a.sin(), a.sin()).normalize()
    })
}

/// Returns a grazing herd of big walkers near one of a few spots of the planet of `globe`, and the
/// universe time it grazes at.
fn grazing(globe: &Arc<Globe>, life: &Life, body: BodyId) -> Option<(Herd, f64)> {
    let spec = Universe::new(SEED).planet(body);
    let land = Land {
        globe,
        life,
        planet: &spec,
    };
    let fits = |h: &Herd| {
        let walker = life
            .species
            .get(usize::from(h.species))
            .is_some_and(big_walker);
        walker && h.state == HerdState::Graze && h.count > 1
    };
    for at in spots() {
        for k in 0..8 {
            let time = START + 400.0 * f64::from(k);
            let search = Search {
                at,
                radius: 2000.0,
                planet: globe.radius(),
                time,
            };
            let herds = read(land, time, |g| near(life, g, &search));
            if let Some(herd) = herds.into_iter().find(fits) {
                return Some((herd, time));
            }
        }
    }
    None
}

/// Returns the planet position of every drawn member of herd `id` in `app`, by slot.
fn members(app: &mut App, id: u64) -> Vec<(u8, DVec3)> {
    let beasts = staying(app).into_iter();
    let herded = beasts.filter(|b| matches!(b.lead, Lead::Herd { id: h, .. } if h == id));
    herded
        .map(|b| (b.lead.key().1, b.spot.planet(b.motion.root.at)))
        .collect()
}

/// Runs `app` for `seconds` and returns how far each drawn member of herd `id` walked meanwhile,
/// read once a second.
fn paths(app: &mut App, id: u64, seconds: u32) -> Vec<f64> {
    let mut last = members(app, id);
    let mut walked = vec![0.0; last.len()];
    for _ in 0..seconds {
        run(app, 1.0, 0.1);
        let now = members(app, id);
        for (i, (slot, was)) in last.iter_mut().enumerate() {
            if let Some((_, at)) = now.iter().find(|(s, _)| *s == *slot) {
                walked[i] += at.distance(*was);
                *was = *at;
            }
        }
    }
    walked
}

/// A grazing herd watched from 100 m, out of sight and earshot, steps about as it feeds: within 30 s
/// at least half of its drawn members walk a metre or more.
#[test]
fn a_grazing_herd_steps_about() {
    let (body, globe, life) = planet(true);
    let life = life.expect("the rig's home planet lives");
    let (herd, time) = grazing(&globe, &life, body).expect("a grazing herd of big walkers");
    let watch = beside(herd.centre, Vec2::new(100.0, 0.0), globe.radius());
    let mut app = stage(globe, body, watch, time);
    run(&mut app, 3.0, 0.1);
    let walked = paths(&mut app, herd.id, 30);
    let moved = walked.iter().filter(|m| **m >= 1.0).count();
    println!(
        "{moved} of {} grazers walked a metre or more: {walked:.1?}",
        walked.len()
    );
    assert!(!walked.is_empty(), "no member of the herd was drawn");
    assert!(
        moved * 2 >= walked.len(),
        "only {moved} of {} grazers stepped",
        walked.len()
    );
}

/// Returns true for a plant eater a robot outweighs enough to make it flee.
fn shy(species: &Species) -> bool {
    plant_eater(species.role.guild) && species.history.mass < ROBOT_KG / PREY_SHARE
}

/// A robot that walks up to a shy plant eater, a metre and a half off, makes it run: within three
/// seconds it is more than five metres from the robot.
#[test]
fn a_shy_animal_runs_from_a_robot() {
    let (mut app, herd) = app_of(true, shy);
    assert!(herd.is_some(), "a herd of a skittish species");
    run(&mut app, 20.0, 0.1);
    let globe = app
        .world()
        .resource::<crate::world::Terrain>()
        .globe()
        .clone();
    let life = globe.life().expect("the rig's planet lives");
    let found = staying(&mut app).into_iter().find(|b| {
        let species = &life.species[usize::from(b.index)];
        matches!(b.lead, Lead::Herd { .. }) && shy(species)
    });
    let beast = found.expect("a skittish herd member");
    put(
        &mut app,
        beast
            .spot
            .planet(beast.motion.root.at + Vec3::new(1.5, 1.0, 0.0)),
    );
    run(&mut app, 3.0, 0.05);
    let me = robot(&mut app);
    let fled = staying(&mut app)
        .into_iter()
        .find(|b| b.lead.key() == beast.lead.key());
    let fled = fled.expect("the animal stays drawn");
    let metres = fled.spot.planet(fled.motion.root.at).distance(me);
    println!("{metres:.1} m from the robot after three seconds");
    assert!(metres > 5.0, "it stayed {metres} m from the robot");
}
