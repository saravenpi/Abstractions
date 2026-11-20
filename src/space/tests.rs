//! Space tests: universe positions carry cells, and the anchor maps frames to render space.

use std::sync::Arc;

use bevy::math::{DQuat, DVec3, IVec3, Vec3};

use super::*;
use crate::universe::{GALAXY_CELL, SECTOR_M, Universe};
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Moving past a cell edge carries into the cell index, moving back a sector carries into the
/// sector and then the cell, and differences stay exact.
#[test]
fn unipos_carries_cells() {
    let a = UniPos::new(IVec3::ZERO, DVec3::new(GALAXY_CELL - 10.0, 50.0, 0.0));
    let b = a + DVec3::new(25.0, -8.0, 0.0);
    assert_eq!(b.cell, IVec3::new(1, 0, 0));
    assert_eq!(b - a, DVec3::new(25.0, -8.0, 0.0));
    assert_eq!(a.distance(b), (25.0f64 * 25.0 + 64.0).sqrt());
    assert_eq!((b - DVec3::new(25.0, -8.0, 0.0)).cell, a.cell);
    let back = b - DVec3::X * SECTOR_M;
    assert_eq!((back.cell.x, back.sector.x), (0, SECTORS_PER_CELL - 1));
    assert!(((b - back).x / SECTOR_M - 1.0).abs() < 1e-12);
}

/// The anchor of the legacy flat world is the identity: render coordinates are world coordinates.
#[test]
fn flat_anchor_is_the_identity() {
    let terrain = Terrain::new(4);
    let spawn = Vec3::new(120.5, 3.25, -640.0);
    let anchor = Anchor::surface(Universe::new(4).home(), &terrain, spawn);
    let p = Vec3::new(-77.5, 12.0, 901.25);
    assert_eq!(anchor.to_render(p.as_dvec3()), p);
    assert_eq!(anchor.to_frame(p), p.as_dvec3());
    assert_eq!(anchor.rotation_to_render(DQuat::IDENTITY), Quat::IDENTITY);
    assert_eq!(altitude(&anchor, terrain.globe(), p), 12.0);
    assert_eq!(up(&anchor, p), Vec3::Y);
}

/// On a globe the anchor renders the anchored spot at its chart coordinates with +Y up.
#[test]
fn globe_anchor_is_upright_at_its_spot() {
    let globe = Arc::new(Globe::new(4, 6000.0, 22.0));
    for face in Face::ALL {
        let terrain = Terrain::on(globe.clone(), globe.face_chart(face)).around(900.0, -350.0);
        let at = Vec3::new(900.0, 7.5, -350.0);
        let anchor = Anchor::surface(Universe::new(4).home(), &terrain, at);
        assert!(!anchor.flat);
        let near = at + Vec3::new(6.0, 2.0, -9.0);
        let seen = anchor.to_render(terrain.embed(near.x, near.y, near.z));
        assert!((seen - near).length() < 0.02, "{face:?} {seen} {near}");
        assert!((up(&anchor, at) - Vec3::Y).length() < 1e-5);
        assert!((altitude(&anchor, &globe, at) - 7.5).abs() < 1e-3);
        assert!((anchor.to_render(anchor.to_frame(near)) - near).length() < 1e-3);
        let turn = anchor.rotation_to_render(anchor.rotation.inverse());
        assert!(turn.angle_between(Quat::IDENTITY) < 1e-5);
    }
}

/// Gravity is the surface value at and below sea level, a quarter of it at two radii, and
/// points down the local vertical.
#[test]
fn gravity_follows_the_inverse_square() {
    let globe = Arc::new(Globe::new(4, 3000.0, 17.5));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::NegZ)).around(10.0, 20.0);
    let anchor = Anchor::surface(
        Universe::new(4).home(),
        &terrain,
        Vec3::new(10.0, 0.0, 20.0),
    );
    let surface = Vec3::new(10.0, 0.0, 20.0);
    let pull = local_gravity(&anchor, &globe, surface);
    assert!((pull.length() - 17.5).abs() < 1e-4);
    assert!((pull.normalize() + up(&anchor, surface)).length() < 1e-6);
    let high = surface + up(&anchor, surface) * 3000.0;
    assert!((local_gravity(&anchor, &globe, high).length() - 17.5 / 4.0).abs() < 1e-3);
    let deep = surface - up(&anchor, surface) * 40.0;
    assert!((local_gravity(&anchor, &globe, deep).length() - 17.5).abs() < 1e-4);
    let flat = Anchor::surface(Universe::new(4).home(), &Terrain::new(4), Vec3::ZERO);
    assert_eq!(
        local_gravity(&flat, &globe, Vec3::new(5.0, 0.0, 5.0)),
        Vec3::NEG_Y * 17.5
    );
}

/// Returns an app that only copies placed poses, a surface anchor on a 6 km globe that is not
/// in the app yet, and a pose near the anchored spot.
fn placing() -> (App, Anchor, Placed) {
    let globe = Arc::new(Globe::new(4, 6000.0, 22.0));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(300.0, 80.0);
    let anchor = Anchor::surface(
        Universe::new(4).home(),
        &terrain,
        Vec3::new(300.0, 4.0, 80.0),
    );
    let pose = Placed {
        pos: terrain.embed(312.0, 9.0, 71.0),
        rot: DQuat::from_rotation_x(0.4),
    };
    let mut app = App::new();
    app.add_systems(Update, frame::sync_placed);
    (app, anchor, pose)
}

/// A placed entity lands at the render pose of its planet-frame pose and keeps its scale. Without
/// an anchor nothing moves.
#[test]
fn placed_poses_reach_transforms() {
    let (mut app, anchor, pose) = placing();
    let scaled = Transform::from_scale(Vec3::splat(2.0));
    let entity = app.world_mut().spawn((pose, scaled)).id();
    app.update();
    assert_eq!(*app.world().get::<Transform>(entity).unwrap(), scaled);
    app.insert_resource(anchor);
    app.update();
    let expected = Transform {
        translation: anchor.to_render(pose.pos),
        rotation: anchor.rotation_to_render(pose.rot),
        scale: Vec3::splat(2.0),
    };
    assert_eq!(*app.world().get::<Transform>(entity).unwrap(), expected);
}

/// A placed entity follows a new anchor and a new pose, and is left alone otherwise.
#[test]
fn placed_poses_follow_changes() {
    let (mut app, anchor, pose) = placing();
    app.insert_resource(anchor);
    let entity = app.world_mut().spawn((pose, Transform::default())).id();
    app.update();
    let moved = rebased(&anchor, 6000.0, Vec3::new(390.0, 0.0, 80.0)).unwrap();
    app.insert_resource(moved);
    app.update();
    let seen = *app.world().get::<Transform>(entity).unwrap();
    assert_eq!(seen.translation, moved.to_render(pose.pos));
    let next = DVec3::new(0.0, 6001.0, 0.0);
    app.world_mut().get_mut::<Placed>(entity).unwrap().pos = next;
    app.update();
    let seen = *app.world().get::<Transform>(entity).unwrap();
    assert_eq!(seen.translation, moved.to_render(next));
    app.world_mut()
        .get_mut::<Transform>(entity)
        .unwrap()
        .translation = Vec3::ONE;
    app.update();
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().translation,
        Vec3::ONE
    );
}

/// The frame systems fit their schedules: a frame and a fixed tick run before any world exists.
#[test]
fn frame_systems_run_without_a_world() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, plugin));
    let entity = app.world_mut().spawn(Transform::default()).id();
    app.update();
    app.world_mut().run_schedule(FixedFirst);
    assert_eq!(
        *app.world().get::<Transform>(entity).unwrap(),
        Transform::default()
    );
}
