//! Rebase tests: an anchor change keeps everything where it was on the planet.

use std::sync::Arc;

use bevy::ecs::system::RunSystemOnce;

use super::*;
use crate::space::up;
use crate::universe::Universe;
use crate::world::Globe;
use crate::world::globe::Face;

/// Sea level radius of the test planet in metres.
const RADIUS: f64 = 3000.0;

/// Returns a terrain on the test planet and a surface anchor near chart position (40, -25).
fn fixture() -> (Terrain, Anchor) {
    let globe = Arc::new(Globe::new(4, RADIUS, 22.0));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(40.0, -25.0);
    let at = Vec3::new(40.0, 2.0, -25.0);
    let anchor = Anchor::surface(Universe::new(4).home(), &terrain, at);
    (terrain, anchor)
}

/// Returns the planet-frame position, linear velocity and angular velocity of a free entity.
fn planet_state(world: &World, entity: Entity, anchor: &Anchor) -> [DVec3; 3] {
    let transform = world.get::<Transform>(entity).unwrap();
    let to_frame = anchor.rotation.inverse();
    [
        anchor.to_frame(transform.translation),
        to_frame * world.get::<LinearVelocity>(entity).unwrap().0.as_dvec3(),
        to_frame * world.get::<AngularVelocity>(entity).unwrap().0.as_dvec3(),
    ]
}

/// Spawns a free physics entity at a render pose with a velocity.
fn spawn_free(world: &mut World, at: Vec3, velocity: Vec3) -> Entity {
    let rot = Quat::from_euler(EulerRot::YXZ, at.x, 0.3, at.z);
    let stale = Position(at - Vec3::splat(0.2));
    let pose = Transform::from_translation(at).with_rotation(rot);
    let speed = (LinearVelocity(velocity), AngularVelocity(velocity.zxy()));
    world
        .spawn((Rebased, pose, stale, Rotation(rot), speed))
        .id()
}

/// Returns a world on the test planet with three free physics entities, the first of them the
/// player body, far enough from the anchored spot for a rebase.
fn scene() -> (World, Vec<Entity>) {
    let (terrain, anchor) = fixture();
    let mut world = World::new();
    world.insert_resource(terrain);
    world.insert_resource(anchor);
    let spots = [
        (Vec3::new(95.0, 3.0, -60.0), Vec3::new(4.0, -9.0, 2.5)),
        (Vec3::new(-20.0, 40.0, 75.0), Vec3::new(-30.0, 12.0, 8.0)),
        (Vec3::new(140.0, -6.0, 10.0), Vec3::new(0.5, 49.0, -17.0)),
    ];
    let free = spots.map(|(at, v)| spawn_free(&mut world, at, v)).to_vec();
    world.entity_mut(free[0]).insert(Player);
    (world, free)
}

/// A rebase moves no free entity on the planet by more than 1 mm, changes no planet-frame
/// velocity by more than 1e-4 m/s, and leaves the physics pose equal to the transform.
#[test]
fn rebase_keeps_planet_frame_state() {
    let (mut world, free) = scene();
    let old = *world.resource::<Anchor>();
    let before = free.iter().map(|&e| planet_state(&world, e, &old));
    let before: Vec<_> = before.collect();
    world.run_system_once(rebase_on_foot).unwrap();
    let new = *world.resource::<Anchor>();
    assert_ne!(new, old);
    for (&entity, was) in free.iter().zip(&before) {
        let now = planet_state(&world, entity, &new);
        assert!(
            now[0].distance(was[0]) < 1e-3,
            "{}",
            now[0].distance(was[0])
        );
        assert!(
            now[1].distance(was[1]) < 1e-4,
            "{}",
            now[1].distance(was[1])
        );
        assert!(
            now[2].distance(was[2]) < 1e-4,
            "{}",
            now[2].distance(was[2])
        );
        let pose = world.get::<Transform>(entity).unwrap();
        assert_eq!(world.get::<Position>(entity).unwrap().0, pose.translation);
        assert_eq!(world.get::<Rotation>(entity).unwrap().0, pose.rotation);
    }
}

/// A rebase leaves children to their parents, and a second run right after one changes nothing.
#[test]
fn rebase_moves_roots_once() {
    let (mut world, free) = scene();
    let local = Transform::from_xyz(0.0, 0.7, 0.0);
    let child = world.spawn((Rebased, local, ChildOf(free[0]))).id();
    world.run_system_once(rebase_on_foot).unwrap();
    assert_eq!(*world.get::<Transform>(child).unwrap(), local);
    let (anchor, pose) = (
        *world.resource::<Anchor>(),
        *world.get::<Transform>(free[1]).unwrap(),
    );
    world.run_system_once(rebase_on_foot).unwrap();
    assert_eq!(*world.resource::<Anchor>(), anchor);
    assert_eq!(*world.get::<Transform>(free[1]).unwrap(), pose);
}

/// After any rebase the anchored spot is upright, sits at the render origin under the body,
/// and every heading keeps its planet-frame direction.
#[test]
fn rebased_anchor_is_upright_and_keeps_headings() {
    let (_, mut anchor) = fixture();
    let mut rebases = 0;
    for step in 0..400 {
        let turn = step as f32 * 0.37;
        let pos = Vec3::new(
            turn.cos() * 9.0 * (step % 7) as f32,
            1.5 + turn.sin(),
            turn.sin() * 31.0,
        );
        let Some(next) = rebased(&anchor, RADIUS, pos) else {
            continue;
        };
        rebases += 1;
        let shift = Shift::between(&anchor, &next);
        assert!((up(&next, Vec3::ZERO) - Vec3::Y).length() < 1e-6);
        assert!((next.origin.length() - RADIUS).abs() < 1e-6);
        let moved = shift.point(pos);
        assert!(moved.xz().length() < 1e-3 && (up(&next, moved) - Vec3::Y).length() < 1e-6);
        let rot = Quat::from_euler(EulerRot::YXZ, turn, 0.2, 0.0);
        let was = anchor.rotation.inverse() * (rot * Vec3::NEG_Z).as_dvec3();
        let now = next.rotation.inverse() * (shift.turn(rot) * Vec3::NEG_Z).as_dvec3();
        assert!(was.distance(now) < 1e-6, "{}", was.distance(now));
        assert_eq!(rebased(&next, RADIUS, moved), None);
        anchor = next;
    }
    assert!(rebases > 100, "{rebases}");
}

/// The limit is 1 percent of the radius between 30 and 300 m, nothing rebases inside it, and
/// the flat test fixture never rebases.
#[test]
fn rebase_waits_for_the_limit() {
    assert_eq!(rebase_limit(1000.0), 30.0);
    assert_eq!(rebase_limit(6000.0), 60.0);
    assert_eq!(rebase_limit(40_000.0), 300.0);
    let (_, start) = fixture();
    let tangent = rebased(&start, RADIUS, Vec3::new(500.0, 0.0, 0.0)).unwrap();
    assert_eq!(rebased(&tangent, RADIUS, Vec3::new(29.0, 250.0, 0.0)), None);
    assert!(rebased(&tangent, RADIUS, Vec3::new(0.0, 4.0, -31.0)).is_some());
    let flat = Anchor::surface(Universe::new(4).home(), &Terrain::new(4), Vec3::ZERO);
    assert_eq!(rebased(&flat, RADIUS, Vec3::new(9000.0, 0.0, 0.0)), None);
}
