//! Tests of the system that aims the water mirror: at a shore with the setting on it poses its
//! camera under the water and tells the water to read it; with the setting off, the sea hidden,
//! no water near or the camera looking up, its camera stays off and the water reads the sky only.

use bevy::camera::Exposure;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_persistent::StorageFormat;

use super::super::Sea;
use super::super::floor::SeaFloor;
use super::super::material::WaterMaterial;
use super::super::uniform::WaterUniform;
use super::aim::aim;
use super::{MirrorCamera, Reflected};
use crate::render::GameCamera;
use crate::settings::{Persistent, Reflections, Settings};

/// Returns settings with the water reflections `value`, kept in a scratch file of this test run.
fn settings(tag: &str, value: Reflections) -> Persistent<Settings> {
    let dir =
        std::env::temp_dir().join(format!("abstractions-mirror-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut settings = Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(dir.join("settings.toml"))
        .default(Settings::default())
        .build()
        .expect("settings in a scratch directory");
    settings.graphics.lighting.reflections = value;
    settings
}

/// Spawns a sea at height 0 under a focus 2 m up, over a full map of water, and returns it.
fn sea(world: &mut World) -> Entity {
    let mut images = Assets::<Image>::default();
    let mut materials = Assets::<WaterMaterial>::default();
    let water = WaterUniform {
        eye: Vec4::new(0.0, 2.0, 0.0, 2.0),
        upward: Vec4::new(0.0, 1.0, 0.0, 0.0),
        deep: Vec4::new(0.0, 0.0, 0.0, 1.0),
        ..default()
    };
    let handle = materials.add(WaterMaterial {
        water,
        floor: Handle::default(),
        place: default(),
        mirror: Handle::default(),
    });
    let floor = SeaFloor::new(&mut images);
    world.insert_resource(materials);
    let sea = (Sea, MeshMaterial3d(handle), floor, Visibility::Inherited);
    world.spawn(sea).id()
}

/// Returns a world with the sea of [`sea`], the robot standing in it 10 m ahead, the game camera
/// 3 m up behind it looking at it from `pitch` radians, and the mirror camera, off. It also
/// returns the sea.
fn shore(value: Reflections, pitch: f32, tag: &str) -> (World, Entity) {
    let mut world = World::new();
    let sea = sea(&mut world);
    world.spawn((Reflected, GlobalTransform::from_xyz(0.0, 1.0, -10.0)));
    let eye = Transform::from_xyz(0.0, 3.0, 5.0).with_rotation(Quat::from_rotation_x(pitch));
    let lens = Projection::Perspective(PerspectiveProjection {
        aspect_ratio: 16.0 / 9.0,
        ..default()
    });
    let light = Exposure { ev100: 11.0 };
    world.spawn((GameCamera, GlobalTransform::from(eye), lens.clone(), light));
    let off = Camera {
        is_active: false,
        ..default()
    };
    let pose = (Transform::default(), GlobalTransform::default());
    world.spawn((
        MirrorCamera(Handle::default()),
        off,
        pose,
        lens,
        Exposure::default(),
    ));
    world.insert_resource(settings(tag, value));
    (world, sea)
}

/// Returns whether the mirror camera draws, where it stands, its exposure, and whether the water
/// reads the mirror image.
fn aimed(world: &mut World, sea: Entity) -> (bool, Vec3, f32, f32) {
    world.run_system_once(aim).expect("the aim runs");
    let mut mirrors = world.query::<(&Camera, &GlobalTransform, &Exposure, &MirrorCamera)>();
    let (camera, pose, exposure, _) = mirrors.single(world).expect("one mirror camera");
    let (active, at, ev) = (camera.is_active, pose.translation(), exposure.ev100);
    let handle = world
        .get::<MeshMaterial3d<WaterMaterial>>(sea)
        .expect("a sea")
        .0
        .clone();
    let strength = world
        .resource::<Assets<WaterMaterial>>()
        .get(&handle)
        .expect("a material")
        .water
        .mirror
        .x;
    (active, at, ev, strength)
}

/// At the shore the mirror camera draws from the eye's reflection under the water, with the
/// exposure of the game camera, and the water reads its image.
#[test]
fn at_a_shore_the_mirror_draws() {
    let (mut world, sea) = shore(Reflections::Actors, -0.15, "on");
    let (active, at, ev, strength) = aimed(&mut world, sea);
    assert!(active && strength == 1.0);
    assert!((at - Vec3::new(0.0, -3.0, 5.0)).length() < 1e-4, "{at}");
    assert_eq!(ev, 11.0);
}

/// With the setting off, the sea hidden, no water near or the camera looking up the mirror camera
/// stays off and the water reads the sky only.
#[test]
fn the_mirror_is_off_without_the_setting_or_water_in_view() {
    let (mut world, sea) = shore(Reflections::Off, -0.15, "off");
    assert!(!aimed(&mut world, sea).0);
    let (mut world, sea) = shore(Reflections::Actors, 1.3, "up");
    let (active, _, _, strength) = aimed(&mut world, sea);
    assert!(!active && strength == 0.0);
    let (mut world, sea) = shore(Reflections::Actors, -0.15, "hidden");
    world.entity_mut(sea).insert(Visibility::Hidden);
    assert!(!aimed(&mut world, sea).0);
    let (mut world, sea) = shore(Reflections::Actors, -0.15, "dry");
    world.get_mut::<SeaFloor>(sea).expect("a floor").wet = 0;
    assert!(!aimed(&mut world, sea).0);
    let (mut world, sea) = shore(Reflections::Actors, -0.15, "again");
    assert!(aimed(&mut world, sea).0);
    world
        .resource_mut::<Persistent<Settings>>()
        .graphics
        .lighting
        .reflections = Reflections::Off;
    let (active, _, _, strength) = aimed(&mut world, sea);
    assert!(!active && strength == 0.0);
}
