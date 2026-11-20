//! The sky dome entity: a huge sphere that follows the camera and feeds the sky shader.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use super::SunDir;
use super::atmosphere::horizon_dip;
use super::celestial::smoothstep;
use super::clouds::CloudCrossing;
use super::galaxy_light::GalaxyLight;
use super::palette::Palette;
use super::sky::{SkyLook, SkyMaterial, SkyUniform};
use super::vantage::{LocalSystem, Vantage};
use super::warp::Lens;
use crate::screens::Screen;

/// Radius of the dome in metres. Its shader writes the far depth, so the dome lies behind the
/// stars and every body at any radius, and the radius only has to hold the camera.
pub const DOME_RADIUS: f32 = 4.0e6;

/// Brightness of the horizon rim against the horizon color at the ground.
const RIM_GAIN: f32 = 0.9;

/// Marks the sky dome.
#[derive(Component, Debug, Clone, Copy)]
pub struct SkyDome;

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
            map: None,
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
    local: Res<'w, LocalSystem>,
    vantage: Res<'w, Vantage>,
    sun: Res<'w, SunDir>,
    crossing: Res<'w, CloudCrossing>,
    lens: Res<'w, Lens>,
    light: Res<'w, GalaxyLight>,
}

impl SkyInputs<'_> {
    /// Returns what the camera's place adds to the palette.
    ///
    /// The rim along the horizon shows once the sky above has thinned by height, as bright as
    /// the air of the planet makes it, and hands over to the halo shell of the planet above the air.
    fn look(&self) -> SkyLook {
        let vantage = &*self.vantage;
        let to_render = vantage.observer.to_render;
        let leaving = smoothstep(1.0, 2.0, vantage.altitude / vantage.atmosphere.max(1.0));
        let share = self.local.air_below(vantage).share();
        let rim = Palette::under(&self.sun, vantage, &self.local).0.horizon * RIM_GAIN;
        SkyLook {
            up: vantage.up,
            dip: horizon_dip(vantage.altitude, vantage.radius),
            sun_height: self.sun.height,
            air: vantage.air,
            whiteout: self.crossing.depth,
            rim: rim * (share - vantage.air) * (1.0 - leaving),
            to_galaxy: Mat3::from_quat(to_render.inverse().as_quat()),
            mapped: self.light.map().is_some(),
            murk: super::murk::under(&self.local, vantage),
            heading: self.lens.heading,
            beta: self.lens.beta,
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
            material.map = inputs.light.map();
        }
    }
}

/// Registers the dome systems; the update runs once the camera is tracked and the clouds are known.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_dome)
        .add_systems(
            PostUpdate,
            update_dome
                .after(super::clouds::drift)
                .after(super::warp::run),
        );
}
