//! The sky dome entity: a big sphere that follows the camera and feeds the sky shader.

use std::f32::consts::TAU;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use super::camera::submerged;
use super::palette::Palette;
use super::sky::{SkyMaterial, SkyUniform};
use super::{DayClock, GameCamera};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};

/// Radius of the dome: past the farthest terrain at 512 meters, inside the camera far plane.
const DOME_RADIUS: f32 = 800.0;

/// Marks the sky dome.
#[derive(Component, Debug, Clone, Copy)]
pub struct SkyDome;

/// Returns how much of the sky the clouds cover, drifting slowly over the day.
pub fn cloud_cover(phase: f32) -> f32 {
    0.55 + 0.12 * (phase * TAU * 3.0).sin()
}

/// Spawns the dome when the world starts loading.
fn spawn_dome(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    palette: Res<Palette>,
) {
    let sky = SkyUniform::new(&palette, 0.0, 0.0, false);
    commands.spawn((
        SkyDome,
        Mesh3d(meshes.add(Sphere::new(DOME_RADIUS).mesh().uv(32, 16))),
        MeshMaterial3d(materials.add(SkyMaterial { sky })),
        Transform::default(),
        NotShadowCaster,
        NotShadowReceiver,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Everything the sky shader values are computed from.
#[derive(SystemParam)]
struct SkyInputs<'w> {
    palette: Res<'w, Palette>,
    clock: Res<'w, DayClock>,
    time: Res<'w, Time>,
    settings: Res<'w, Persistent<Settings>>,
}

/// Centers the dome on the camera and refreshes the shader values.
fn update_dome(
    inputs: SkyInputs,
    cameras: Query<&GlobalTransform, With<GameCamera>>,
    mut domes: Query<(&mut Transform, &MeshMaterial3d<SkyMaterial>), With<SkyDome>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    let cover = match inputs.settings.graphics.clouds {
        true => cloud_cover(inputs.clock.phase),
        false => 0.0,
    };
    let sky = SkyUniform::new(
        &inputs.palette,
        inputs.time.elapsed_secs(),
        cover,
        submerged(camera),
    );
    for (mut transform, handle) in &mut domes {
        transform.translation = camera.translation();
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.sky = sky;
        }
    }
}

/// Registers the dome systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_dome)
        .add_systems(Update, update_dome);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cloud cover stays within a sensible band over the whole cycle.
    #[test]
    fn cloud_cover_stays_in_range() {
        for step in 0..100 {
            let cover = cloud_cover(step as f32 / 100.0);
            assert!((0.4..=0.7).contains(&cover));
        }
    }
}
