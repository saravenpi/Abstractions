//! The starfield: the real stars near the camera and the far galaxies, as one mesh on a far shell.

mod build;
mod specks;

#[cfg(test)]
mod tests;

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::palette::Palette;
use super::vantage::Vantage;
use crate::screens::Screen;
use crate::universe::GalaxyId;

/// Radius of the star shell in metres: behind every body, in front of the dome.
pub const STAR_SHELL: f32 = 3.0e6;

/// Distance in game light-years up to which stars are drawn.
pub(super) const REACH_GLY: f64 = 40.0;

/// Most stars drawn, the nearest ones.
pub(super) const MOST_STARS: usize = 4000;

/// Galaxy grid cells searched for galaxies around the camera's own.
pub(super) const GALAXY_CELLS: i32 = 2;

/// Uniform block read by `starfield.wesl`.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct StarUniform {
    /// Visibility of the stars from 0 to 1 in x; the rest is unused.
    pub params: Vec4,
}

/// Material of the starfield.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct StarMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub stars: StarUniform,
}

impl Material for StarMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/starfield.wesl".into()
    }

    /// Starlight adds to the sky behind it.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// The stars stay out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }
}

/// The starfield entity, with the place its mesh was last built for.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Starfield {
    /// Galaxy and position in metres from its centre of the last build, if any.
    pub built: Option<(GalaxyId, DVec3)>,
}

/// Spawns the starfield without a mesh when the world starts loading.
fn spawn(mut commands: Commands, mut materials: ResMut<Assets<StarMaterial>>) {
    commands.spawn((
        Starfield::default(),
        MeshMaterial3d(materials.add(StarMaterial::default())),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Keeps the shell around the camera, turned with the galaxy, and fades the stars with the sky.
fn follow(
    vantage: Res<Vantage>,
    palette: Res<Palette>,
    mut fields: Query<
        (
            &mut Transform,
            &mut GlobalTransform,
            &MeshMaterial3d<StarMaterial>,
        ),
        With<Starfield>,
    >,
    mut materials: ResMut<Assets<StarMaterial>>,
) {
    let pose = Transform {
        translation: vantage.camera,
        rotation: vantage.observer.to_render.as_quat(),
        scale: Vec3::splat(STAR_SHELL),
    };
    for (mut transform, mut global, handle) in &mut fields {
        super::pin(pose, &mut transform, &mut global);
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.stars.params.x = palette.stars;
        }
    }
}

/// Embeds the shader, registers the material and runs the starfield once the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "starfield.wesl");
    app.add_plugins(MaterialPlugin::<StarMaterial>::default())
        .add_systems(OnEnter(Screen::Loading), spawn)
        .add_systems(
            PostUpdate,
            (build::adopt, build::launch, follow)
                .chain()
                .after(super::sun::track),
        );
}
