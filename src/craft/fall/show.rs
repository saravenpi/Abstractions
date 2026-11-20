//! The cache shown where it lies, and taken back when the robot comes to it.

use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::cache::Left;
use crate::avatar::hurt::Scars;
use crate::craft::Pack;
use crate::player::Player;
use crate::screens::Screen;
use crate::space::{Anchor, Frame, Rebased};

/// Distance from the cache within which the robot takes it back, in metres.
const PICK: f32 = 2.0;

/// Size of the drawn cache: width, height and depth, in metres.
const SIZE: Vec3 = Vec3::new(0.5, 0.3, 0.4);

/// Marks the drawn cache.
#[derive(Component)]
pub struct Waiting;

/// Run condition: a cache lies on the planet the anchor is on and is not drawn yet.
pub fn wanted(
    left: Res<Left>,
    anchor: Option<Res<Anchor>>,
    waiting: Query<(), With<Waiting>>,
) -> bool {
    let here = left
        .0
        .as_ref()
        .zip(anchor)
        .is_some_and(|(cache, anchor)| anchor.frame == Frame::Surface(cache.body));
    here && waiting.is_empty()
}

/// Draws the cache as a crate lying where the robot fell.
pub fn show(mut commands: Commands, (left, anchor, scars): (Res<Left>, Res<Anchor>, Res<Scars>)) {
    let Some(cache) = left.0.as_ref() else { return };
    let at = anchor.to_render(DVec3::from(cache.at));
    commands.spawn((
        Waiting,
        Mesh3d(scars.cube.clone()),
        MeshMaterial3d(scars.panel.clone()),
        Transform::from_translation(at + Vec3::Y * SIZE.y * 0.5).with_scale(SIZE),
        Visibility::default(),
        NotShadowCaster,
        Rebased,
        DespawnOnExit(Screen::Gameplay),
    ));
}

/// Takes the cache back into the pack when the robot stands next to it.
pub fn recover(
    mut commands: Commands,
    (mut left, mut pack): (ResMut<Left>, ResMut<Pack>),
    player: Query<&Transform, With<Player>>,
    waiting: Query<(Entity, &Transform), With<Waiting>>,
) {
    let (Ok(body), Ok((entity, crate_at))) = (player.single(), waiting.single()) else {
        return;
    };
    if body.translation.distance(crate_at.translation) > PICK {
        return;
    }
    if let Some(cache) = left.0.take() {
        for stack in cache.pack.stacks {
            pack.stow(stack, f32::MAX);
        }
        for item in cache.pack.items {
            pack.keep(item.recipe, item.materials);
        }
    }
    commands.entity(entity).despawn();
}
