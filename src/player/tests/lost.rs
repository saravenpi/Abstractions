//! The rescue of a body under the ground: once, onto the collider, and held still where no
//! collider is loaded yet, never falling and coming back in a loop.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::stage::{app, body};
use crate::player::asks::gather;
use crate::player::controller::drive;
use crate::player::lost::{Rescues, Stranded, plugin};
use crate::player::spawn::Home;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player, PlayerSet};
use crate::sim::PlayerId;
use crate::space::{Anchor, Location, Travel};
use crate::universe::BodyId;
use crate::view::Cut;
use crate::world::Terrain;

/// Returns a stage whose flat ground stands at the height the planet gives under the body, with
/// the controller and the rescue on, and that height, run for a few frames so the ground's
/// collider is where it was moved. The app is already finished, so the rescue is added by its
/// function, not as a plugin.
fn stage() -> (App, f32) {
    let mut app = app(&[(PlayerId::LOCAL, 0.0)]);
    app.add_message::<Cut>()
        .insert_resource(Location {
            body: BodyId::default(),
            travel: Travel::OnFoot,
        })
        .add_systems(Update, (gather, drive).chain().in_set(PlayerSet::Move));
    plugin(&mut app);
    let anchor = *app.world().resource::<Anchor>();
    let terrain = app.world().resource::<Terrain>().clone();
    let chart = terrain.locate(anchor.to_frame(Vec3::ZERO));
    let height = terrain.height(chart.x, chart.z);
    let world = app.world_mut();
    let mut grounds = world.query_filtered::<&mut Transform, (With<RigidBody>, Without<Player>)>();
    let mut bodies = world.query_filtered::<Entity, With<Player>>();
    let player = bodies.single(world).expect("one body");
    for mut ground in grounds.iter_mut(world) {
        ground.translation.y = height - 0.5;
    }
    let home = Home(anchor.to_frame(Vec3::Y * (height + 1.0)));
    let start = Transform::from_translation(Vec3::Y * standing(height));
    world.entity_mut(player).insert((home, start));
    for _ in 0..3 {
        app.update();
    }
    (app, height)
}

/// Sinks the body 10 m under `height`, at rest.
fn sink(app: &mut App, height: f32) {
    let world = app.world_mut();
    let mut bodies = world.query_filtered::<&mut Transform, With<Player>>();
    for mut transform in bodies.iter_mut(world) {
        transform.translation = Vec3::Y * (height - 10.0);
    }
}

/// Returns the height of the centre of a body standing on ground at `height`.
fn standing(height: f32) -> f32 {
    height + BODY_LENGTH * 0.5 + BODY_RADIUS + 0.05
}

/// Returns true while the body waits for a collider.
fn stranded(app: &mut App) -> bool {
    let world = app.world_mut();
    let mut held = world.query_filtered::<(), (With<Player>, With<Stranded>)>();
    held.iter(world).next().is_some()
}

/// A body sunk under loaded ground is stood on the collider at once, rescued once, and stays.
#[test]
fn a_body_under_the_ground_is_stood_on_the_collider_once() {
    let (mut app, height) = stage();
    sink(&mut app, height);
    for _ in 0..30 {
        app.update();
    }
    let (transform, _, _) = body(&mut app, PlayerId::LOCAL);
    let (wanted, at) = (standing(height), transform.translation.y);
    assert!((at - wanted).abs() < 0.1, "{at} for {wanted}");
    let rescues = *app.world().resource::<Rescues>();
    assert_eq!((rescues.count, rescues.repeats), (1, 0));
    assert!(!stranded(&mut app));
}

/// A body sunk where no collider is loaded is held still on the height the planet gives, not
/// rescued again, and stood on the collider once it loads.
#[test]
fn a_body_with_no_collider_under_it_waits_then_stands() {
    let (mut app, height) = stage();
    let world = app.world_mut();
    let mut grounds = world.query_filtered::<Entity, (With<RigidBody>, Without<Player>)>();
    let ground: Vec<Entity> = grounds.iter(world).collect();
    for entity in &ground {
        world.entity_mut(*entity).insert(ColliderDisabled);
    }
    app.update();
    sink(&mut app, height);
    for _ in 0..60 {
        app.update();
        let (transform, motion, _) = body(&mut app, PlayerId::LOCAL);
        let held = transform.translation.y;
        assert!(held > height, "{held} under {height}");
        assert_eq!(motion.velocity, Vec3::ZERO);
    }
    assert!(stranded(&mut app));
    let world = app.world_mut();
    for entity in &ground {
        world.entity_mut(*entity).remove::<ColliderDisabled>();
    }
    for _ in 0..5 {
        app.update();
    }
    let (transform, _, _) = body(&mut app, PlayerId::LOCAL);
    assert!((transform.translation.y - standing(height)).abs() < 0.1);
    assert_eq!(app.world().resource::<Rescues>().count, 1);
    assert!(!stranded(&mut app));
}
