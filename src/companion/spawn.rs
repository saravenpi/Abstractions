//! Spawning the companion beside the player: its meshes, materials, picture and parts.

use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::Companion;
use super::body::{SCREEN_FRONT, body};
use super::brain::Brain;
use super::face::Frame;
use super::face::machine::Face;
use super::limbs::{HAND, Hand, THRUSTER, Thruster};
use super::motion::{Float, Head, Rig};
use super::orb::{orb, screen};
use super::screen::{Display, picture};
use super::shine::Lamp;
use super::tune::{HALF, LIGHT_RANGE, SIDE};
use crate::render::{GameCamera, Warm};
use crate::screens::Screen;
use crate::space::Rebased;

/// Radius of the shared orb in meters: the antenna tip as it is, the hands and the thruster scaled from it.
const ORB_RADIUS: f32 = 0.03;

/// Where the antenna ends, above and slightly behind the top of the head.
const ANTENNA_TIP: Vec3 = Vec3::new(0.0, HALF[1] + 0.11, 0.025);

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
}

/// Returns an unlit forward material, which shows its base colour as it is and so can glow through bloom.
fn unlit(picture: Option<Handle<Image>>) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::BLACK,
        base_color_texture: picture,
        unlit: true,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Returns the matte lit material of the shell and the hands, coloured by the vertex colours.
fn matte() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
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
/// glowing parts, and on the body the hands and the thruster.
fn dress(rig: &mut ChildSpawnerCommands, stock: &mut Stock, display: &Display) {
    let shell = MeshMaterial3d(stock.materials.add(matte()));
    let ball = Mesh3d(stock.meshes.add(orb(ORB_RADIUS)));
    let parts = (
        Mesh3d(stock.meshes.add(body(ANTENNA_TIP))),
        Mesh3d(stock.meshes.add(screen())),
    );
    rig.spawn((Head, Transform::default(), Visibility::default()))
        .with_children(|head| {
            head.spawn((parts.0, shell.clone()));
            let face = MeshMaterial3d(display.face.clone());
            head.spawn((parts.1, face, NotShadowCaster));
            glows(head, ball.clone(), display);
        });
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
}

/// Hands one of each mesh layout and material of the companion to the pipeline warm-up. It
/// only appears once play begins, after the warm-up ran.
pub fn warm(mut warm: ResMut<Warm>, mut stock: Stock) {
    let display = display(&mut stock);
    let shell = stock.materials.add(matte());
    warm.add(stock.meshes.add(body(ANTENNA_TIP)), shell);
    warm.add(stock.meshes.add(screen()), display.face);
    warm.add(stock.meshes.add(orb(ORB_RADIUS)), display.glow);
}

/// Spawns the companion beside the player's eyes when there is none. It leaves with the title screen.
pub fn spawn(
    mut commands: Commands,
    mut stock: Stock,
    camera: Single<&GlobalTransform, With<GameCamera>>,
    existing: Query<(), With<Companion>>,
) {
    if !existing.is_empty() {
        return;
    }
    let seed = u64::from(stock.time.elapsed_secs().to_bits());
    let at = camera.translation() + camera.right() * SIDE;
    let display = display(&mut stock);
    let mut bot = commands.spawn((
        Name::new("Companion"),
        Companion,
        Rebased,
        Brain::new(seed),
        Face::new(seed ^ 0xFACE),
        Float::default(),
        Transform::from_translation(at).looking_at(camera.translation(), camera.up()),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
    ));
    bot.with_children(|root| {
        root.spawn((Rig, Transform::default(), Visibility::default()))
            .with_children(|rig| dress(rig, &mut stock, &display));
    });
    bot.insert(display);
}
