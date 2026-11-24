//! Tests of follower synchronization: smooth walker and flyer tracking, snapping on large gaps,
//! and folding displaced remote creatures into scan wants.

use std::time::Duration;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::beast::Beast;
use super::rig::{START, planet};
use super::senses::Stance;
use super::stage::{run, stage, staying};
use super::sync::{RemoteFauna, SyncTarget, fold_wants};
use super::tune::DRIFT;
use super::want::{From, Lead};
use crate::life::Life;
use crate::life::body::Locomotion;
use crate::life::herd::Ledger;
use crate::space::Clock;
use crate::world::Terrain;

/// Returns the first living species matching `locomotion` in `life`.
fn pick_species(life: &Life, locomotion: Locomotion) -> usize {
    life.species
        .iter()
        .position(|s| s.body.locomotion == locomotion && !s.body.parts.is_empty())
        .expect("living planet has species for requested locomotion")
}

/// Spawns a test `Beast` in `app` for `species` at local `at` and returns its herd key.
fn add_beast(app: &mut App, species: usize, key: (u64, u8), local: Vec3) {
    let terrain = app.world().resource::<Terrain>().clone();
    let life = terrain.globe().life().unwrap();
    let sp = &life.species[species];
    let spot = super::place::Spot::at(DVec3::Y, terrain.globe().radius());
    let ground_y = spot.ground(&terrain, local.xz());
    let at = Vec3::new(local.x, ground_y + local.y, local.z);
    let birth = super::beast::Birth {
        lead: Lead::Herd {
            id: key.0,
            slot: key.1,
        },
        species: (sp, species as u16),
        at: (spot, at, Vec3::NEG_Z),
        now: 0.0,
    };
    let entity = super::beast::spawn(&mut app.world_mut().commands(), birth, &[], &[]);
    app.update();
    assert!(app.world().get::<Beast>(entity).is_some());
}

/// A follower walker converges smoothly onto a moving target and keeps planted feet on the ground.
#[test]
fn synced_walker_tracks_moving_target_smoothly_and_stays_grounded() {
    let (body, globe, life) = planet(true);
    let life = life.expect("living planet");
    let walker_idx = pick_species(&life, Locomotion::Walk);
    let mut app = stage(globe.clone(), body, DVec3::Y, START);
    add_beast(&mut app, walker_idx, (501, 0), Vec3::ZERO);

    let dt = 1.0 / 30.0;
    let total = 2.5_f32;
    let steps = (total / dt).round() as u32;
    let speed = Vec3::new(2.2, 0.0, -1.4);
    let terrain = app.world().resource::<Terrain>().clone();
    for step in 1..=steps {
        let now = START + f64::from(step) * f64::from(dt);
        let spot = staying(&mut app)[0].spot;
        let step_pos = speed * (step as f32 * dt);
        let y = spot.ground(&terrain, step_pos.xz());
        let planet_pos = spot.planet(Vec3::new(step_pos.x, y, step_pos.z));
        let vel = (spot.rot * speed.as_dvec3()).as_vec3();
        let heading = vel.normalize();
        app.world_mut()
            .resource_mut::<RemoteFauna>()
            .targets
            .insert(
                (501, 0),
                SyncTarget {
                    species: walker_idx as u16,
                    pos: planet_pos,
                    vel,
                    heading,
                    stance: Stance::Wary,
                    excite: 0.4,
                    time: now,
                },
            );
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        app.world_mut().resource_mut::<Clock>().seconds = now;
        app.update();
    }

    let beast = staying(&mut app)
        .into_iter()
        .find(|b| b.lead.key() == (501, 0))
        .unwrap();
    let target = app
        .world()
        .resource::<RemoteFauna>()
        .get((501, 0))
        .unwrap()
        .pos;
    let pos = beast.spot.planet(beast.motion.root.place);
    let err = (pos - target).length();
    assert!(err < 0.45, "walker lagged by {err} m");
    assert_eq!(beast.wits.stance, Stance::Wary);
    for foot in beast.motion.feet.iter().filter(|f| f.planted) {
        let gap = (foot.at.y - beast.spot.ground(&terrain, foot.at.xz())).abs();
        assert!(gap < 0.05, "foot {gap} m off ground");
    }
}

/// A flyer converges in 3D onto an elevated moving target, and large discrepancies snap cleanly.
#[test]
fn synced_flyer_tracks_3d_and_far_targets_snap_and_respot() {
    let (body, globe, life) = planet(true);
    let life = life.expect("living planet");
    let flyer_idx = life
        .species
        .iter()
        .position(|s| {
            matches!(s.body.locomotion, Locomotion::Fly | Locomotion::Swarm)
                && !s.body.parts.is_empty()
        })
        .expect("living planet has a flyer");
    let mut app = stage(globe.clone(), body, DVec3::Y, START);
    add_beast(&mut app, flyer_idx, (777, 1), Vec3::new(0.0, 20.0, 0.0));

    let terrain = app.world().resource::<Terrain>().clone();
    let beast = staying(&mut app)
        .into_iter()
        .find(|b| b.lead.key() == (777, 1))
        .unwrap();
    let spot = beast.spot;
    let climb_y = spot.ground(&terrain, Vec2::new(2.0, -3.0)) + 22.0;
    let climb_target = spot.planet(Vec3::new(2.0, climb_y, -3.0));
    let vel = (spot.rot * Vec3::new(1.0, 0.5, -1.0).as_dvec3()).as_vec3();
    app.world_mut()
        .resource_mut::<RemoteFauna>()
        .targets
        .insert(
            (777, 1),
            SyncTarget {
                species: flyer_idx as u16,
                pos: climb_target,
                vel,
                heading: vel.normalize(),
                stance: Stance::Calm,
                excite: 0.2,
                time: START,
            },
        );
    run(&mut app, 2.0, 1.0 / 30.0);
    let beast = staying(&mut app)
        .into_iter()
        .find(|b| b.lead.key() == (777, 1))
        .unwrap();
    let got = beast.spot.planet(beast.motion.root.at);
    let predicted = climb_target + vel.as_dvec3() * 0.20;
    assert!(
        got.distance(predicted) < 2.5,
        "flyer 3d dist={}",
        got.distance(predicted)
    );

    let old_origin = beast.spot.origin;
    let far_y = spot.ground(&terrain, Vec2::new(DRIFT + 40.0, 0.0)) + 25.0;
    let far_pos = spot.planet(Vec3::new(DRIFT + 40.0, far_y, 0.0));
    let t = app.world().resource::<Clock>().seconds;
    app.world_mut()
        .resource_mut::<RemoteFauna>()
        .targets
        .insert(
            (777, 1),
            SyncTarget {
                species: flyer_idx as u16,
                pos: far_pos,
                vel: Vec3::ZERO,
                heading: (spot.rot * Vec3::X.as_dvec3()).as_vec3(),
                stance: Stance::Attack,
                excite: 1.0,
                time: t,
            },
        );
    run(&mut app, 0.1, 1.0 / 30.0);
    let moved = staying(&mut app)
        .into_iter()
        .find(|b| b.lead.key() == (777, 1))
        .unwrap();
    assert_ne!(moved.spot.origin, old_origin);
    assert!(moved.spot.planet(moved.motion.root.at).distance(far_pos) < 0.5);
}

/// `fold_wants` injects living remotely owned beasts within reach and omits dead ones in `Ledger`.
#[test]
fn fold_wants_injects_live_remote_creatures_and_skips_dead() {
    let (_, globe, life) = planet(true);
    let life = life.expect("living planet");
    let walker_idx = pick_species(&life, Locomotion::Walk) as u16;
    let radius = globe.radius();
    let spot = super::place::Spot::at(DVec3::Y, radius);

    let mut remote = RemoteFauna::default();
    remote.targets.insert(
        (88, 0),
        SyncTarget {
            species: walker_idx,
            pos: spot.planet(Vec3::new(20.0, 0.0, 0.0)),
            vel: Vec3::ZERO,
            heading: Vec3::NEG_Z,
            stance: Stance::Calm,
            excite: 0.0,
            time: START,
        },
    );
    remote.targets.insert(
        (88, 1),
        SyncTarget {
            species: walker_idx,
            pos: spot.planet(Vec3::new(15.0, 0.0, 0.0)),
            vel: Vec3::ZERO,
            heading: Vec3::NEG_Z,
            stance: Stance::Calm,
            excite: 0.0,
            time: START,
        },
    );

    let mut ledger = Ledger::default();
    ledger.kill(88, 1, START - 10.0);
    let from = From {
        robot: DVec3::Y,
        time: START,
        radius,
    };
    let mut wants = Vec::new();
    fold_wants(&remote, (&life, &[], &ledger), from, &mut wants);
    assert_eq!(wants.len(), 1);
    assert_eq!(wants[0].lead.key(), (88, 0));
}
