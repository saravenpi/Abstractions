//! The boot nozzles: a flare under each boot while the boosters push.

use std::f32::consts::PI;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::kit::Kit;
use super::scene::{SOLES, Scene};
use crate::gear::Gear;
use crate::player::Player;
use crate::screens::Screen;
use crate::space::Rebased;

/// Sideways distance of each boot from the body axis, in metres.
const STANCE: f32 = 0.13;

/// Width and length of a flare at full push, in metres.
const SIZE: Vec2 = Vec2::new(0.16, 0.55);

/// Flicker rate in radians per second.
const FLICKER: f32 = 47.0;

/// One nozzle flare, free in render space. The number is its side: -1 left, 1 right.
#[derive(Component, Debug, Clone, Copy)]
pub struct Flare(pub f32);

/// Spawns the two hidden flares when gameplay starts, unless they already exist.
pub fn spawn(mut commands: Commands, kit: Res<Kit>, old: Query<(), With<Flare>>) {
    if !old.is_empty() {
        return;
    }
    for side in [-1.0, 1.0] {
        commands.spawn((
            Flare(side),
            Mesh3d(kit.cone.clone()),
            MeshMaterial3d(kit.flare.clone()),
            Transform::from_scale(Vec3::ZERO),
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
}

/// Gives every player the gear. Both pieces of equipment are owned from the start of the game.
pub fn equip(mut commands: Commands, bare: Query<Entity, (With<Player>, Without<Gear>)>) {
    for player in &bare {
        commands.entity(player).insert(Gear::default());
    }
}

/// Returns the pose of the flare on `side` in the body model frame, at `push` from 0 to 1,
/// `seconds` into the game.
///
/// The cone hangs tip down from the sole, grows with the push and flickers.
pub fn pose(side: f32, push: f32, seconds: f32) -> Transform {
    let flicker = 0.85 + 0.15 * (seconds * FLICKER + side * 1.7).sin();
    let length = SIZE.y * push * flicker;
    Transform {
        translation: Vec3::new(side * STANCE, -SOLES - length * 0.5, 0.0),
        rotation: Quat::from_rotation_x(PI),
        scale: Vec3::new(SIZE.x, length, SIZE.x) * push.min(1.0).sqrt(),
    }
}

/// Places the flares under the boots of the body as the gear poses it, sized to the booster
/// push. They rest at zero scale while the boosters are off.
pub fn draw(time: Res<Time>, scene: Res<Scene>, mut flares: Query<(&Flare, &mut Transform)>) {
    let push = scene.push();
    for (flare, mut transform) in &mut flares {
        if push <= 0.0 && transform.scale == Vec3::ZERO {
            continue;
        }
        let local = pose(flare.0, push, time.elapsed_secs());
        *transform = Transform {
            translation: scene.at(local.translation),
            rotation: scene.rot * local.rotation,
            scale: local.scale,
        };
    }
}
