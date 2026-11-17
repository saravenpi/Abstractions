//! The sky dome entity: a huge sphere that follows the camera and feeds the sky shader.

use std::f32::consts::TAU;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::math::DVec3;
use bevy::prelude::*;

use super::atmosphere::horizon_dip;
use super::celestial::smoothstep;
use super::clouds::CloudCrossing;
use super::palette::Palette;
use super::sky::{SkyLook, SkyMaterial, SkyUniform};
use super::vantage::Vantage;
use super::{DayClock, SunDir};
use crate::screens::Screen;

/// Radius of the dome in metres: behind the stars and every body, which are drawn nearer.
pub const DOME_RADIUS: f32 = 4.0e6;

/// Brightness of the horizon rim against the horizon color at the ground.
const RIM_GAIN: f32 = 0.9;

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
) {
    commands.spawn((
        SkyDome,
        Mesh3d(meshes.add(super::shell(false))),
        MeshMaterial3d(materials.add(SkyMaterial {
            sky: SkyUniform::default(),
        })),
        Transform::from_scale(Vec3::splat(DOME_RADIUS)),
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
    vantage: Res<'w, Vantage>,
    sun: Res<'w, SunDir>,
    crossing: Res<'w, CloudCrossing>,
}

impl SkyInputs<'_> {
    /// Returns what the camera's place adds to the palette.
    ///
    /// The rim along the horizon shows once the sky above has thinned, and hands over to the halo
    /// shell of the planet above the air.
    fn look(&self) -> SkyLook {
        let vantage = &*self.vantage;
        let to_render = vantage.observer.to_render;
        let leaving = smoothstep(1.0, 2.0, vantage.altitude / vantage.atmosphere.max(1.0));
        let rim = Palette::at(self.clock.phase).horizon * RIM_GAIN;
        SkyLook {
            up: vantage.up,
            dip: horizon_dip(vantage.altitude, vantage.radius),
            sun_height: self.sun.height,
            air: vantage.air,
            whiteout: self.crossing.depth,
            rim: rim * (1.0 - vantage.air) * (1.0 - leaving),
            disc: (to_render * DVec3::Y).as_vec3(),
            core: (to_render * (-vantage.galactic).normalize_or_zero()).as_vec3(),
            band: vantage.band,
            submerged: vantage.altitude < 0.0,
        }
    }
}

/// Centers the dome on the camera and refreshes the shader values.
fn update_dome(
    inputs: SkyInputs,
    mut domes: Query<
        (
            &mut Transform,
            &mut GlobalTransform,
            &MeshMaterial3d<SkyMaterial>,
        ),
        With<SkyDome>,
    >,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let sky = SkyUniform::new(&inputs.palette, &inputs.look());
    let pose =
        Transform::from_translation(inputs.vantage.camera).with_scale(Vec3::splat(DOME_RADIUS));
    for (mut transform, mut global, handle) in &mut domes {
        super::pin(pose, &mut transform, &mut global);
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.sky = sky;
        }
    }
}

/// Registers the dome systems; the update runs once the camera is tracked and the clouds are known.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_dome)
        .add_systems(PostUpdate, update_dome.after(super::clouds::drift));
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
