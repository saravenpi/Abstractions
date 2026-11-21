//! The rescue of a body under the ground: once, onto the collider, and held still where no
//! collider is loaded yet, never falling and coming back in a loop.

use std::sync::Arc;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::stage::{app, body};
use crate::player::asks::gather;
use crate::player::controller::drive;
use crate::player::lost::{Rescues, Stranded, plugin};
use crate::player::spawn::Home;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player, PlayerSet};
use crate::sim::{Intent, IntentOf, PlayerId};
use crate::space::{Anchor, Location, Travel};
use crate::universe::{BodyId, PlanetClass, PlanetRecord, STANDARD_GRAVITY, Water};
use crate::view::Cut;
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// A dry world, so the ground under the body is never under a sea it would swim in.
const DESERT: PlanetRecord = PlanetRecord {
    class: PlanetClass::Desert,
    pressure: 0.4,
    water: Water::None,
    ocean: 0.0,
    ..PlanetRecord::TEMPERATE
};

/// Returns a stage on a dry world, on the chart of the face that holds the body, with the
/// controller and the rescue on, and the height the planet gives under the body. The flat
/// ground stands at that height and the body on it, and a few frames have run so the ground's
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
    let globe = Arc::new(Globe::new(4, 3000.0, STANDARD_GRAVITY).with_record(DESERT));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    app.insert_resource(terrain.clone());
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

/// Right after a rescue the body stands on the ground at rest, nothing left of the fall, and
/// walks off at once: a second ahead takes it metres over the ground, still on its feet.
#[test]
fn a_rescued_body_stands_and_walks() {
    let (mut app, height) = stage();
    sink(&mut app, height);
    app.update();
    let (start, motion, gear) = body(&mut app, PlayerId::LOCAL);
    assert!(
        motion.grounded && motion.velocity == Vec3::ZERO,
        "{motion:?}"
    );
    assert!(!gear.state.mode.airborne() && !motion.wings);
    for _ in 0..30 {
        let intent = Intent::Move {
            forward: 1.0,
            strafe: 0.0,
        };
        let player = PlayerId::LOCAL;
        app.world_mut().write_message(IntentOf { player, intent });
        app.update();
    }
    let (end, motion, _) = body(&mut app, PlayerId::LOCAL);
    let walked = end
        .translation
        .with_y(0.0)
        .distance(start.translation.with_y(0.0));
    assert!(
        motion.grounded && walked > 2.0,
        "walked {walked} m, {motion:?}"
    );
    assert_eq!(app.world().resource::<Rescues>().count, 1);
}
