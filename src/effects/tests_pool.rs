//! Tests of the particle pool against the frame systems: every particle follows a rebase.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::pool::{self, POOL_SIZE, Particle};
use crate::space::Rebased;

/// Every pooled particle is a free root entity marked `Rebased`, so a rebase leaves it in place.
#[test]
fn every_particle_follows_rebases() {
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>();
    let world = app.world_mut();
    world
        .run_system_once(pool::build)
        .expect("the particle looks");
    world.run_system_once(pool::spawn).expect("the particles");
    let all = world
        .query_filtered::<(), With<Particle>>()
        .iter(world)
        .count();
    let free = world
        .query_filtered::<(), (With<Particle>, With<Rebased>, Without<ChildOf>)>()
        .iter(world)
        .count();
    assert_eq!((all, free), (POOL_SIZE, POOL_SIZE));
}
