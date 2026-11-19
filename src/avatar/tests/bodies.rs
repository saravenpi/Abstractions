//! Several bodies at once: each animates from its own input, with or without a player.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::app::{Forced, Free, count, world};
use super::scene;
use crate::avatar::animate::{Anim, Bone};
use crate::avatar::pose::Core;
use crate::avatar::rig::SEGS;
use crate::avatar::spawn::{Lamp, Plume};
use crate::avatar::{AvatarPose, Looks, Mode, Stock, puppet};

/// Returns the animation of the body that hangs from a player and of the body `puppet`, which
/// must hang from nothing.
fn posed(app: &mut App, puppet: Entity) -> (Anim, Anim) {
    let mut bodies = app.world_mut().query::<(Entity, &Anim, Has<ChildOf>)>();
    let all: Vec<_> = bodies.iter(app.world()).collect();
    assert_eq!(all.len(), 2);
    let find = |own: bool| all.iter().find(|(entity, ..)| (*entity != puppet) == own);
    let (own, other) = (find(true).expect("a body"), find(false).expect("a puppet"));
    assert!(own.2 && !other.2, "the puppet hangs from no player");
    (*own.1, *other.1)
}

/// Two bodies with different inputs take different poses in the same frame, and a body with
/// no player and no controller, spawned from the shared look, animates from the input it is given.
#[test]
fn every_body_animates_from_its_own_input() {
    let (mut app, _) = world();
    let puppet = app
        .world_mut()
        .run_system_once(
            |mut commands: Commands, mut stock: Stock, looks: Res<Looks>| {
                let root = puppet(&mut commands, &mut stock, &looks, None);
                commands.entity(root).insert(Free);
                root
            },
        )
        .expect("the look is shared once a body exists");
    app.insert_resource(Forced(scene(Mode::Idle, 0.0)));
    *app.world_mut()
        .get_mut::<AvatarPose>(puppet)
        .expect("a pose") = scene(Mode::Glide, 1.0);
    (0..60).for_each(|_| app.update());
    let (own, other) = posed(&mut app, puppet);
    let pitch = |anim: &Anim| anim.pose.core(Core::Pitch);
    assert!(
        pitch(&other) > 1.0 && pitch(&own) < 0.2,
        "puppet {} own {}",
        pitch(&other),
        pitch(&own)
    );
    assert!(super::gap(&own.pose, &other.pose) > 0.5);
    assert_eq!(count::<Bone>(&mut app), 2 * SEGS);
    assert_eq!((count::<Plume>(&mut app), count::<Lamp>(&mut app)), (4, 2));
}
