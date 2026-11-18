//! The starfield: the real stars near the camera and the far galaxies, as one mesh on a far shell,
//! and the dust a warp rushes through, as a second fixed mesh drawn with the same material.

mod build;
mod depth;
mod dust;
mod material;
mod specks;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_depth;

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;

pub use material::{StarMaterial, StarUniform};

use super::palette::Palette;
use super::vantage::Vantage;
use super::warp::{LAPS, Lens};
use crate::screens::Screen;
use crate::universe::{GLY, GalaxyId};
use dust::Dust;

/// Radius of the star shell in metres: behind every body, in front of the dome.
pub const STAR_SHELL: f32 = 3.0e6;

/// Distance in game light-years up to which stars are drawn.
pub(super) const REACH_GLY: f64 = 40.0;

/// Most stars drawn, the nearest ones.
pub(super) const MOST_STARS: usize = 4000;

/// Galaxy grid cells searched for galaxies around the camera's own.
pub(super) const GALAXY_CELLS: i32 = 2;

/// The starfield entity, with the place its mesh was last built for.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Starfield {
    /// Galaxy and position in metres from its centre of the last build, if any.
    pub built: Option<(GalaxyId, DVec3)>,
}

impl Starfield {
    /// Returns how far a camera at `vantage` is from the place the mesh was built for, in game
    /// light-years in galaxy axes: the shader slides every star by it. Without a mesh, or with
    /// the mesh of another galaxy, it is nothing.
    pub fn drift(&self, vantage: &Vantage) -> Vec3 {
        match self.built {
            Some((galaxy, at)) if galaxy == vantage.galaxy => {
                ((vantage.galactic - at) / GLY).as_vec3()
            }
            _ => Vec3::ZERO,
        }
    }
}

/// Spawns the starfield without a mesh when the world starts loading, and the dust with its
/// fixed mesh and the same material, hidden until a warp.
fn spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<StarMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let material = MeshMaterial3d(materials.add(StarMaterial::default()));
    let sky = (
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    );
    commands.spawn((Starfield::default(), material.clone(), sky.clone()));
    commands.spawn((Dust, Mesh3d(meshes.add(dust::mesh())), material, sky));
}

/// The dust entity: where it is drawn and whether it shows.
type DustPose<'a> = (
    &'a mut Transform,
    &'a mut GlobalTransform,
    &'a mut Visibility,
);

/// Keeps the shell around the camera, turned with the galaxy, fades the stars with the sky and
/// hands the shader the warp lens: the same stars crowd ahead, streak, sway with the flow and
/// come back to rest, and the dust shows only while a warp has some in.
fn follow(
    vantage: Res<Vantage>,
    (palette, lens): (Res<Palette>, Res<Lens>),
    mut fields: Query<(
        &Starfield,
        &mut Transform,
        &mut GlobalTransform,
        &MeshMaterial3d<StarMaterial>,
    )>,
    mut dust: Query<DustPose, (With<Dust>, Without<Starfield>)>,
    mut materials: ResMut<Assets<StarMaterial>>,
) {
    let pose = Transform {
        translation: vantage.camera,
        rotation: vantage.observer.to_render.as_quat(),
        scale: Vec3::splat(STAR_SHELL),
    };
    for (field, mut transform, mut global, handle) in &mut fields {
        super::pin(pose, &mut transform, &mut global);
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.stars = StarUniform {
                params: Vec4::new(palette.stars, lens.streak, lens.stars, 0.0),
                heading: lens.heading.extend(lens.beta),
                drift: field.drift(&vantage).extend(0.0),
                flow: Vec4::new(lens.phase, lens.flow, lens.dust, lens.flow / LAPS),
            };
        }
    }
    let shown = [Visibility::Hidden, Visibility::Visible][usize::from(lens.dust > 0.0)];
    for (mut transform, mut global, mut visibility) in &mut dust {
        super::pin(pose, &mut transform, &mut global);
        visibility.set_if_neq(shown);
    }
}

/// Embeds the shader, registers the material and runs the starfield once the camera is tracked
/// and the warp sequence has moved on.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "starfield.wesl");
    app.add_plugins(MaterialPlugin::<StarMaterial>::default())
        .add_systems(OnEnter(Screen::Loading), spawn)
        .add_systems(
            PostUpdate,
            (build::adopt, build::launch, follow)
                .chain()
                .after(super::sun::track)
                .after(super::warp::run),
        );
}
