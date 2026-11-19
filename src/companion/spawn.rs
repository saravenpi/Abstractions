//! Spawning the companion beside the player: its meshes, materials, picture and parts.

use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Companion;
use super::body::SCREEN_FRONT;
use super::brain::Brain;
use super::face::Frame;
use super::face::machine::Face;
use super::limbs::{HAND, Hand, THRUSTER, Thruster};
use super::motion::{Float, Head, Rig};
use super::puppet::{Parts, Tracked};
use super::screen::{Display, picture};
use super::shapes::{ANTENNA_TIP, Shapes, unlit};
use super::shine::Lamp;
use super::tune::{HALF, LIGHT_RANGE, SIDE};
use super::{ribbon, tail};
use crate::render::{GameCamera, Warm};
use crate::screens::Screen;
use crate::space::Rebased;

/// How far in front of the screen the light sits, in meters, so it does not burn a spot on the bezel.
const LAMP_REACH: f32 = 0.16;

/// The asset stores the companion's parts are created in, and the clock that seeds it.
#[derive(SystemParam)]
pub struct Stock<'w> {
    /// Mesh assets.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// Material assets.
    materials: ResMut<'w, Assets<StandardMaterial>>,
    /// Image assets.
    images: ResMut<'w, Assets<Image>>,
    /// Time, whose elapsed seconds seed the companion's choices.
    time: Res<'w, Time>,
    /// The shared meshes and materials, once a companion was made.
    shapes: Option<Res<'w, Shapes>>,
}

/// Creates the picture and the materials whose colours follow the face.
fn display(stock: &mut Stock) -> Display {
    let image = stock.images.add(picture(&Frame::default()));
    Display {
        face: stock.materials.add(unlit(Some(image.clone()))),
        glow: stock.materials.add(unlit(None)),
        image,
        shown: None,
        tint: Vec3::ZERO,
        thrust: 0.0,
    }
}

/// Spawns the glowing parts of the head as its children: antenna tip and light.
fn glows(head: &mut ChildSpawnerCommands, ball: Mesh3d, display: &Display) {
    let glow = MeshMaterial3d(display.glow.clone());
    let front = Transform::from_translation(Vec3::NEG_Z * (SCREEN_FRONT + LAMP_REACH));
    let tip = Transform::from_translation(ANTENNA_TIP);
    head.spawn((ball, glow, tip, NotShadowCaster));
    let light = PointLight {
        intensity: 0.0,
        range: LIGHT_RANGE,
        radius: 0.06,
        shadow_maps_enabled: false,
        ..default()
    };
    head.spawn((Lamp, light, front));
}

/// Spawns everything visible as children of the rig: the head with its shell, screen and
/// glowing parts, and on the body the hands and the thruster. Returns the head.
fn dress(rig: &mut ChildSpawnerCommands, shapes: &Shapes, display: &Display) -> Entity {
    let shell = MeshMaterial3d(shapes.shell.clone());
    let ball = Mesh3d(shapes.orb.clone());
    let head = rig
        .spawn((Head, Transform::default(), Visibility::default()))
        .with_children(|head| {
            head.spawn((Mesh3d(shapes.body.clone()), shell.clone()));
            let face = MeshMaterial3d(display.face.clone());
            head.spawn((Mesh3d(shapes.screen.clone()), face, NotShadowCaster));
            glows(head, ball.clone(), display);
        })
        .id();
    for side in [1.0, -1.0] {
        let hand = Hand {
            side,
            lag: Vec3::ZERO,
        };
        rig.spawn((
            hand,
            ball.clone(),
            shell.clone(),
            Transform::from_scale(HAND),
        ));
    }
    let under = Transform::from_xyz(0.0, -HALF[1] - 0.03, 0.0).with_scale(THRUSTER);
    let glow = MeshMaterial3d(display.glow.clone());
    rig.spawn((Thruster, ball, glow, under, NotShadowCaster));
    head
}

/// Spawns the look of a companion at the render pose `at`, with no brain, and returns its
/// root: the shell, the screen, the hands, the thruster, the light, the flying pose and the
/// trail, all driven by the pose of the root alone. The local companion is this plus a brain.
///
/// The meshes and the shell come from the shared [`Shapes`], built on the first call. Only
/// what a companion changes by itself is its own: its picture, its two tinted materials and
/// the mesh of its trail.
pub fn look(commands: &mut Commands, stock: &mut Stock, at: Transform) -> Entity {
    let shapes = match stock.shapes.as_deref() {
        Some(shapes) => shapes.clone(),
        None => Shapes::new(&mut stock.meshes, &mut stock.materials),
    };
    let display = display(stock);
    let puppet = (Float::default(), Tracked::default(), Rebased);
    let root = commands
        .spawn((Name::new("Companion"), Companion, puppet, at))
        .insert((Visibility::default(), DespawnOnEnter(Screen::Title)))
        .id();
    let rig = commands
        .spawn((Rig, Transform::default(), Visibility::default()))
        .insert(ChildOf(root))
        .id();
    let mut head = rig;
    commands
        .entity(rig)
        .with_children(|rig| head = dress(rig, &shapes, &display));
    commands.entity(root).insert((display, Parts { rig, head }));
    tail::spawn(commands, &mut stock.meshes, shapes.trail.clone(), root);
    commands.insert_resource(shapes);
    root
}

/// Hands one of each mesh layout and material of the companion to the pipeline warm-up,
/// the ribbon of its trail included. It only appears once play begins, after the warm-up ran.
pub fn warm(mut warm: ResMut<Warm>, mut stock: Stock) {
    let display = display(&mut stock);
    let shapes = Shapes::new(&mut stock.meshes, &mut stock.materials);
    warm.add(shapes.body.clone(), shapes.shell.clone());
    warm.add(shapes.screen.clone(), display.face);
    warm.add(shapes.orb.clone(), display.glow);
    warm.add(stock.meshes.add(ribbon::mesh()), shapes.trail.clone());
}

/// Spawns the local companion beside the player's eyes when there is none: the shared look,
/// and a brain and a face to drive it. It leaves with the title screen.
pub fn spawn(
    mut commands: Commands,
    mut stock: Stock,
    camera: Single<&GlobalTransform, With<GameCamera>>,
    existing: Query<(), (With<Companion>, With<Brain>)>,
) {
    if !existing.is_empty() {
        return;
    }
    let seed = u64::from(stock.time.elapsed_secs().to_bits());
    let at = camera.translation() + camera.right() * SIDE;
    let at = Transform::from_translation(at).looking_at(camera.translation(), camera.up());
    let root = look(&mut commands, &mut stock, at);
    commands
        .entity(root)
        .insert((Brain::new(seed), Face::new(seed ^ 0xFACE)));
}
