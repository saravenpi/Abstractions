//! Tests of the cost of the pools: the drops are reused in turn and the foam never grows.

use bevy::prelude::*;

use super::drops::{BUBBLES, Drops};
use super::foam::{Foam, Quad, VERTICES, lay, mesh};
use super::tune::{DROPS, QUADS};

/// Returns drops with one entity per slot.
fn pool() -> Drops {
    let mut world = World::new();
    let entities = (0..DROPS).map(|_| world.spawn_empty().id()).collect();
    Drops {
        entities,
        ..Drops::default()
    }
}

/// Taking more drops than the pool holds comes back to the first, and each kind stays in its part.
#[test]
fn take_wraps_the_pool() {
    let mut drops = pool();
    let spray = DROPS - BUBBLES;
    let first = drops.take(false);
    (1..spray).for_each(|_| assert!(drops.take(false).is_some()));
    assert_eq!(drops.take(false), first);
    let bubble = drops.take(true);
    assert_eq!(bubble, drops.entities.get(spray).copied());
    (1..BUBBLES).for_each(|_| assert!(drops.take(true).is_some()));
    assert_eq!(drops.take(true), bubble);
    assert!(drops.awake);
}

/// An empty pool gives nothing and stays asleep.
#[test]
fn an_empty_pool_gives_nothing() {
    let mut drops = Drops::default();
    assert_eq!(drops.take(false), None);
    assert!(!drops.awake);
}

/// Laying many more quads than the pool holds never changes the vertex count of the mesh.
#[test]
fn foam_keeps_a_fixed_vertex_count() {
    let mut meshes = Assets::<Mesh>::default();
    let mut foam = Foam {
        mesh: meshes.add(mesh()),
        ..Foam::default()
    };
    let quad = Quad {
        ends: [Vec3::ZERO, Vec3::X],
        dirs: [Vec3::Z; 2],
        shape: Vec4::new(0.0, 1.0, 0.5, 0.1),
        births: [1.0, 2.0],
        fade: Vec2::new(1.0, 4.0),
    };
    for _ in 0..QUADS * 3 + 5 {
        lay(&mut meshes, &mut foam, &quad);
    }
    let laid = meshes.get(&foam.mesh).map(Mesh::count_vertices);
    assert_eq!(laid, Some(VERTICES));
    assert_eq!(foam.next, 5);
    assert_eq!(foam.until, 6.0);
}
