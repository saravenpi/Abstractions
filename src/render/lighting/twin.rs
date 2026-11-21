//! The twin lights of the water mirror: a copy of the sun and of the moon on the mirror layer
//! only, with no shadow.
//!
//! Bevy gives a view the directional lights whose layers meet its own, and builds shadow maps per
//! view and light. The sun and the moon stay on layer 0, so the mirror camera under the water
//! neither sees them nor renders their cascades again; their twins light what it draws instead.

use bevy::light::DirectionalLight;
use bevy::prelude::*;

use super::{Moon, Sun};
use crate::render::mirror_only;
use crate::screens::Screen;

/// A copy of the sun or of the moon that lights only what the water mirror draws.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Twin {
    /// Whether it copies the moon; else it copies the sun.
    pub moon: bool,
}

/// A source light: its pose, its light and whether it is the moon.
type Source = (
    &'static GlobalTransform,
    &'static DirectionalLight,
    Has<Moon>,
);

/// A twin light: its marker, its poses and its light.
type Copied = (
    &'static Twin,
    &'static mut Transform,
    &'static mut GlobalTransform,
    &'static mut DirectionalLight,
);

/// Spawns the twins of the sun and of the moon when the world starts loading.
pub(super) fn spawn(mut commands: Commands) {
    for moon in [false, true] {
        commands.spawn((
            Twin { moon },
            DirectionalLight {
                shadow_maps_enabled: false,
                ..default()
            },
            mirror_only(),
            Transform::default(),
            Visibility::default(),
            DespawnOnEnter(Screen::Title),
        ));
    }
}

/// Gives each twin the pose, the colour and the strength its source has this frame. The twin
/// never casts a shadow.
pub(super) fn follow(
    sources: Query<Source, (Or<(With<Sun>, With<Moon>)>, Without<Twin>)>,
    mut twins: Query<Copied, (Without<Sun>, Without<Moon>)>,
) {
    for (global, light, moon) in &sources {
        for (twin, mut transform, mut pose, mut copy) in &mut twins {
            if twin.moon == moon {
                *pose = *global;
                *transform = global.compute_transform();
                copy.color = light.color;
                copy.illuminance = light.illuminance;
            }
        }
    }
}
