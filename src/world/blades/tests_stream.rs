//! Tests of the streaming of the bands of blades: a walker never sees bare ground near it, and a rebase of the
//! anchor keeps every band where it stands.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::fade::reach;
use super::tiles::{Eye, Meadow, Spot, Tile, seen};
use super::tune::{FLIGHTS, LANDS};
use crate::render::GameCamera;
use crate::space::{Anchor, Shift, rebased};
use crate::universe::Universe;
use crate::world::Terrain;
use crate::world::globe::{Face, Globe};

/// Frames a band takes from its start to its landing: two frames, though its pool builds it in a few ms.
const LATE: u32 = 2;

/// Frames per second of the walk: the 30 of a loaded Apple M2.
const FPS: f32 = 30.0;

/// Metres from the camera within which every band whose blades stand must be held.
const NEAR: f32 = 25.0;

/// Runs one frame of the meadow for a camera at `eye`, building the bands the way `tend` does, at most
/// [`LANDS`] landed and started a frame and [`FLIGHTS`] at once, each landing [`LATE`] frames after its start.
fn frame(
    meadow: &mut Meadow,
    eye: Eye,
    building: &mut HashMap<Spot, u32>,
    now: u32,
) -> Vec<(f32, Spot)> {
    let (mut wanted, mut gone) = (Vec::new(), Vec::new());
    meadow.plan(eye, &mut wanted, &mut gone);
    building
        .retain(|spot, _| wanted.iter().any(|w| w.1 == *spot) || meadow.tiles.contains_key(spot));
    let mut done: Vec<Spot> = building
        .iter()
        .filter(|(_, at)| **at + LATE <= now)
        .map(|(s, _)| *s)
        .collect();
    done.sort_by_key(|(key, band)| (key.x, key.y, *band));
    for spot in done.into_iter().take(LANDS) {
        building.remove(&spot);
        let tile = Tile {
            entity: None,
            stale: false,
        };
        meadow.tiles.insert(spot, tile);
    }
    let free = wanted
        .iter()
        .filter(|w| !building.contains_key(&w.1) && !meadow.tiles.contains_key(&w.1));
    let starts: Vec<Spot> = free
        .map(|w| w.1)
        .take(LANDS.min(FLIGHTS - building.len().min(FLIGHTS)))
        .collect();
    building.extend(starts.into_iter().map(|spot| (spot, now)));
    wanted
}

/// Returns how many bands whose blades stand within [`NEAR`] of the camera are missing at the worst frame of a
/// walk of `seconds` at `speed` metres per second, after a first fill of six seconds standing.
fn walk(speed: f32, seconds: f32) -> usize {
    let (mut meadow, mut building) = (Meadow::default(), HashMap::new());
    let ahead = Vec2::from_angle(0.3);
    let (fill, frames) = ((6.0 * FPS) as u32, (seconds * FPS) as u32);
    let mut worst = 0;
    for now in 0..fill + frames {
        let walked = now.saturating_sub(fill) as f32 / FPS * speed;
        let eye = Eye::new(Vec2::new(5.0, 3.0) + ahead * walked, 1.7);
        let wanted = frame(&mut meadow, eye, &mut building, now);
        let bare = wanted
            .iter()
            .filter(|(away, (_, band))| *away < NEAR && *away < reach(*band));
        let missing = bare
            .filter(|(_, spot)| !meadow.tiles.contains_key(spot))
            .count();
        worst = worst.max(missing * usize::from(now >= fill));
    }
    worst
}

/// At a walk of 6 m/s, and at the 8.5 m/s of a sprint, every band within 25 m whose blades stand is held at
/// every frame of a minute.
#[test]
fn a_walker_never_sees_bare_ground() {
    for speed in [6.0, 8.5] {
        let missing = walk(speed, 60.0);
        assert_eq!(
            missing, 0,
            "{missing} bands missing near a walker at {speed} m/s"
        );
    }
}

/// Returns a world on a planet of 30 km whose anchor stands at chart position (40, -25), with a player body far
/// enough from it for a rebase and the game camera at its eyes, their global transforms those of the last frame,
/// and the body and the camera.
fn scene() -> (World, Entity, Entity) {
    let globe = Arc::new(Globe::new(4, 30_000.0, 9.81));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(40.0, -25.0);
    let anchor = Anchor::surface(
        Universe::new(4).home(),
        &terrain,
        Vec3::new(40.0, 2.0, -25.0),
    );
    let mut world = World::new();
    world.insert_resource(terrain);
    world.insert_resource(anchor);
    let body = Transform::from_xyz(420.0, 3.0, -60.0);
    let eyes = Transform::from_xyz(0.0, 1.6, 0.0);
    let parent = world.spawn((body, GlobalTransform::from(body))).id();
    let camera = (
        GameCamera,
        eyes,
        GlobalTransform::from(body * eyes),
        ChildOf(parent),
    );
    let camera = world.spawn(camera).id();
    (world, parent, camera)
}

/// Returns where `tend` sees the camera of `world`.
fn eye_of(world: &mut World) -> Vec3 {
    let look = |t: Res<Terrain>,
                a: Res<Anchor>,
                h: TransformHelper,
                c: Query<Entity, With<GameCamera>>| {
        seen(&t, &a, &h, c.single().expect("a camera"))
    };
    world
        .run_system_once(look)
        .expect("runs")
        .expect("a camera pose")
}

/// Holds in `meadow` every band a camera at `at` wants, each with an entity of `world`.
fn fill(meadow: &mut Meadow, world: &mut World, at: Vec3) {
    let (mut wanted, mut gone) = (Vec::new(), Vec::new());
    meadow.plan(Eye::new(at.xz(), 1.7), &mut wanted, &mut gone);
    for (_, spot) in wanted {
        let tile = Tile {
            entity: Some(world.spawn_empty().id()),
            stale: false,
        };
        meadow.tiles.insert(spot, tile);
    }
}

/// Rebases the anchor of `world` under `body` as the frame systems do in `FixedFirst`: the anchor and the
/// transform of the body change, the global transforms stay those of the frame before. Returns where the global
/// transform of `camera` would put it under the new anchor.
fn rebase(world: &mut World, (body, camera): (Entity, Entity)) -> Vec3 {
    let old = *world.resource::<Anchor>();
    let radius = world.resource::<Terrain>().globe().radius();
    let pose = *world.get::<Transform>(body).expect("a body");
    let next = rebased(&old, radius, pose.translation).expect("far enough for a rebase");
    let shift = Shift::between(&old, &next);
    let moved = Transform {
        translation: shift.point(pose.translation),
        rotation: shift.turn(pose.rotation),
        ..pose
    };
    world.entity_mut(body).insert(moved);
    world.insert_resource(next);
    let last = world.get::<GlobalTransform>(camera).expect("a pose");
    world
        .resource::<Terrain>()
        .locate(next.to_frame(last.translation()))
}

/// On the frame of a rebase, before the transforms are propagated, the camera is seen where it stands, so the
/// meadow drops and wants no band; the global transform of the last frame would put it hundreds of metres off.
#[test]
fn a_rebase_keeps_the_bands() {
    let (mut world, body, camera) = scene();
    let before = eye_of(&mut world);
    let mut meadow = Meadow::default();
    fill(&mut meadow, &mut world, before);
    let held = meadow.tiles.len();
    let stale = rebase(&mut world, (body, camera));
    let after = eye_of(&mut world);
    assert!(
        after.distance(before) < 0.01,
        "moved {} m",
        after.distance(before)
    );
    assert!(
        stale.distance(before) > 100.0,
        "the old pose is only {} m off",
        stale.distance(before)
    );
    let (mut wanted, mut gone) = (Vec::new(), Vec::new());
    meadow.plan(Eye::new(after.xz(), 1.7), &mut wanted, &mut gone);
    assert!(
        gone.is_empty() && wanted.is_empty(),
        "{} dropped, {} wanted",
        gone.len(),
        wanted.len()
    );
    assert_eq!(meadow.tiles.len(), held);
}
