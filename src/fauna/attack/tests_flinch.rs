//! Tests of the flinch on a dead body: a part hit as an animal dies, or cut from its carcass, stays
//! where the carcass lies instead of drifting away from it.

use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::Dead;
use super::gesture::{Flinch, gesture};
use crate::fauna::beast::Beast;
use crate::fauna::rig::app;
use crate::fauna::stage::run;

/// A dead body whose part was struck keeps that part where it lies over half a second, and its flinch
/// is dropped.
#[test]
fn a_dead_body_does_not_flinch_apart() {
    let (mut app, _) = app(true);
    run(&mut app, 1.0, 0.1);
    let world = app.world_mut();
    let mut beasts = world.query::<(Entity, &Beast)>();
    let (entity, part) = beasts
        .iter(world)
        .find_map(|(e, b)| Some((e, b.parts.first().copied().flatten()?)))
        .expect("a drawn animal with a part");
    let push = Flinch {
        part: 0,
        push: Vec3::Y * 0.3,
        age: 0.0,
    };
    world.entity_mut(entity).insert((Dead, push));
    let before = *world.get::<Transform>(part).expect("the part");
    for _ in 0..15 {
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 30.0));
        world.run_system_once(gesture).expect("the flinches run");
    }
    let after = *world.get::<Transform>(part).expect("the part");
    let moved = after.translation.distance(before.translation);
    assert!(moved < 1e-6, "the part of the dead body drifted {moved} m");
    assert!(
        world.get::<Flinch>(entity).is_none(),
        "the dead body kept its flinch"
    );
}
