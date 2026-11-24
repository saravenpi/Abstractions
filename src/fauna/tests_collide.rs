//! Tests of creature collisions against solid obstacles, each other and robots.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::math::{DVec3, Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::Kind;
use super::beast::Beast;
use super::bulk::capsule;
use super::collide::math::{Actor, depenetrate_offset, push_from_robot, separate_pair};
use super::collide::resolve;
use super::fade::Fade;
use super::motion::Motion;
use super::place::Spot;
use super::react::{Mood, Temper};
use super::rig::planet;
use super::senses::Wits;
use super::want::Lead;
use crate::life::Life;
use crate::life::body::Locomotion;
use crate::player::{BODY_RADIUS, Player};
use crate::space::{Anchor, Location, Travel};
use crate::world::Terrain;
use crate::world::dither::Way;
use crate::world::globe::Face;

/// Finds the first drawn walking species index in `life`.
fn walker_index(life: &Life) -> usize {
    life.species
        .iter()
        .position(|s| s.body.locomotion == Locomotion::Walk)
        .expect("living planet has a walking species")
}

/// Spawns a bare `Beast` in `app` stepping in `spot` with local `pos` and `vel`.
fn spawn_walker(
    app: &mut App,
    spot: Spot,
    life: &Life,
    species: usize,
    (pos, vel): (Vec3, Vec3),
) -> Entity {
    let spec = &life.species[species];
    let mut motion = Motion::new(&spec.body, pos, Vec3::X, 11);
    motion.root.vel.at = vel;
    let beast = Beast {
        lead: Lead::Herd { id: 1, slot: 0 },
        index: species as u16,
        spot,
        motion,
        parts: Vec::new(),
        mood: Mood::default(),
        temper: Temper::of(spec),
        wits: Wits::of(spec),
        fade: Fade::new(Way::In, 0.0),
    };
    app.world_mut().spawn(beast).id()
}

/// Spawns a static wall box and a static terrain trimesh in `app`, and steps once to index them.
fn add_solids(app: &mut App, spot: Spot, base_y: f32) {
    let anchor = *app.world().resource::<Anchor>();
    let rot = anchor.rotation_to_render(spot.rot);
    let wall_pos = anchor.to_render(spot.planet(Vec3::new(3.0, base_y + 2.0, 0.0)));
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(2.0, 4.0, 10.0),
        Transform::from_translation(wall_pos).with_rotation(rot),
    ));
    let tri_pos = anchor.to_render(spot.planet(Vec3::new(-8.0, base_y + 1.0, 0.0)));
    let floor = Collider::trimesh(
        vec![Vec3::ZERO, Vec3::X * 2.0, Vec3::Z * 2.0],
        vec![[0, 1, 2]],
    );
    app.world_mut().spawn((
        RigidBody::Static,
        floor,
        Transform::from_translation(tri_pos).with_rotation(rot),
    ));
    (0..3).for_each(|_| app.update());
}

/// Builds a minimal physics app on a real planet for running `collide::resolve`.
fn physics_app() -> (App, Terrain, Spot, Life, usize) {
    let (body, globe, life) = planet(true);
    let life = life.expect("living planet");
    let idx = walker_index(&life);
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let spot = Spot::at(DVec3::Y, globe.radius());
    let base_y = spot.ground(&terrain, Vec2::ZERO);
    let anchor = Anchor::surface(body, &terrain, Vec3::new(0.0, base_y, 0.0));
    let step = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0));
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .add_plugins(PhysicsPlugins::default())
        .init_asset::<Mesh>()
        .insert_resource(step)
        .insert_resource(terrain.clone())
        .insert_resource(anchor)
        .insert_resource(Location {
            body,
            travel: Travel::OnFoot,
        })
        .add_systems(Update, resolve);
    app.finish();
    app.cleanup();
    let robot = anchor.to_render(spot.planet(Vec3::new(0.0, base_y, 20.0)));
    app.world_mut()
        .spawn((Player, Transform::from_translation(robot)));
    add_solids(&mut app, spot, base_y);
    (app, terrain, spot, life, idx)
}

/// Non-aloft creatures slide along static wall colliders, ignore trimesh terrain and escape overlap.
#[test]
fn walkers_slide_along_solids_and_escape_overlap() {
    let (mut app, terrain, spot, life, idx) = physics_app();
    let base_y = spot.ground(&terrain, Vec2::ZERO);
    let body = &life.species[idx].body;
    let edge = 2.0 - capsule(body.length, body.height).0.max(0.2);
    let slide_at = (
        Vec3::new(edge + 0.05, base_y, -3.0),
        Vec3::new(6.0, 0.0, 3.0),
    );
    let slider = spawn_walker(&mut app, spot, &life, idx, slide_at);
    let embedded = spawn_walker(
        &mut app,
        spot,
        &life,
        idx,
        (Vec3::new(2.3, base_y, 3.0), Vec3::ZERO),
    );
    let near_tri = spawn_walker(
        &mut app,
        spot,
        &life,
        idx,
        (Vec3::new(-7.5, base_y, 0.5), Vec3::ZERO),
    );
    app.update();
    let bs = app.world().get::<Beast>(slider).unwrap();
    assert!(
        bs.motion.root.at.x <= edge + 1e-2,
        "x={}",
        bs.motion.root.at.x
    );
    assert!(bs.motion.root.vel.at.x <= 1e-3 && bs.motion.root.vel.at.z > 2.5);
    let be = app.world().get::<Beast>(embedded).unwrap();
    assert!(
        be.motion.root.at.x <= edge + 1e-2,
        "unstuck={}",
        be.motion.root.at.x
    );
    let bt = app.world().get::<Beast>(near_tri).unwrap();
    assert!((bt.motion.root.at.x - (-7.5)).abs() < 1e-3);
    assert!(depenetrate_offset(Vec3::ZERO, Vec3::new(-0.3, 0.0, 0.0), true, 0.4, Vec2::X).x < -0.6);
}

/// Pairwise separation weights pushes by inverse mass and lets flyers pass overhead unblocked.
#[test]
fn overlapping_beasts_separate_by_mass_while_flyers_clear_walkers() {
    let (_, globe, life) = planet(true);
    let life = life.unwrap();
    let spot = Spot::at(DVec3::Y, globe.radius());
    let ground_a = Actor {
        entity: Entity::from_raw_u32(1).unwrap(),
        species: 0,
        shift: Vec2::ZERO,
        pos: spot.planet(Vec3::ZERO),
        rad: 1.0,
        r_min: globe.radius(),
        r_max: globe.radius() + 1.5,
        mass: 50.0,
        spot,
    };
    let heavy_b = Actor {
        entity: Entity::from_raw_u32(2).unwrap(),
        pos: spot.planet(Vec3::new(1.0, 0.0, 0.0)),
        mass: 450.0,
        ..ground_a
    };
    let (pa, pb) = separate_pair(&ground_a, &heavy_b).expect("overlapping ground pair");
    assert!(pa.x < 0.0 && pb.x > 0.0);
    assert!((pa.length() / pb.length() - 9.0).abs() < 1e-3);
    let flyer = Actor {
        r_min: globe.radius() + 3.0,
        r_max: globe.radius() + 4.0,
        pos: spot.planet(Vec3::new(0.5, 3.5, 0.0)),
        ..heavy_b
    };
    assert!(
        separate_pair(&ground_a, &flyer).is_none(),
        "{}",
        life.species.len()
    );
}

/// Both light and heavy creatures stay outside the robot capsule while remaining inside bite reach.
#[test]
fn heavy_and_light_beasts_stop_outside_player_within_bite_reach() {
    let (_, globe, life) = planet(true);
    let life = life.unwrap();
    let spot = Spot::at(DVec3::Y, globe.radius());
    let mut small = life.species[walker_index(&life)].body.clone();
    small.length = 0.7;
    small.height = 0.5;
    let mut giant = small.clone();
    giant.length = 3.2;
    giant.height = 1.9;
    let robot = spot.planet(Vec3::ZERO);
    for body in [small, giant] {
        let mut at = Vec3::new(0.25, 0.0, 0.0);
        let push = push_from_robot(&spot, at, &body, robot).expect("inside robot capsule");
        at += Vec3::new(push.x, 0.0, push.y);
        let min_gap = 0.5 * body.length + BODY_RADIUS;
        assert!(at.xz().length() >= min_gap - 1e-4);
        assert!(at.xz().length() < Kind::Bite.reach(&body));
    }
}
