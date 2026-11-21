//! The starfield: the real stars near the camera, as one mesh on a far shell,
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
#[cfg(test)]
mod tests_sight;

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;

pub use material::{StarMaterial, StarUniform};
#[cfg(test)]
pub use specks::DAY_FLUX;

use super::palette::Palette;
use super::skylight;
use super::vantage::Vantage;
use super::warp::Lens;
use crate::screens::Screen;
use crate::universe::{GalaxyId, LY};
use dust::Dust;

/// Radius of the star shell in metres. The shader writes the far depth for every speck, so the
/// stars lie behind every body at any radius.
pub const STAR_SHELL: f32 = 3.0e6;

/// Distance in light-years up to which stars are gathered into the mesh. It is farther
/// than the 40 they are drawn to (`SIGHT` in `starfield.wesl`), by more than the camera moves
/// between two builds, so a camera that has moved since the build still has every star it can
/// see. A star fades in over the last 2.5 of those 40 as the camera closes on it, two and a half
/// seconds of the cruise at one light-year a second, so none pops into the sky when the next mesh
/// is swapped in. A sphere of this radius holds 2,800 stars at the density of the Sun's place, 0.004
/// per cubic light-year, and 8,400 where a sector is full.
pub(super) const REACH_LY: f64 = 55.0;

/// Most stars gathered, the nearest ones: more than a sphere of [`REACH_LY`] holds anywhere but
/// in the saturated core.
pub(super) const MOST_STARS: usize = 9000;

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
                ((vantage.galactic - at) / LY).as_vec3()
            }
            _ => Vec3::ZERO,
        }
    }
}

/// Returns the share of the light of its star that reaches a camera `distance` metres from it,
/// against the light at the planet it is at, whose centre is `orbit` metres from the star, in a
/// system that reaches `reach` metres out.
///
/// The light falls off with the square of the distance: ten times as far out it is one
/// hundredth. Closer in than the planet it stays what the planet gets. Over the outer half of
/// the reach of the system it fades to nothing, so between the stars no star lights the ship
/// as a sun: they are all just stars.
pub fn sunlight(orbit: f64, distance: f64, reach: f64) -> f32 {
    let square = (orbit / distance.max(1.0e-9)).powi(2).min(1.0);
    let out = ((distance / reach - 0.5) / 0.5).clamp(0.0, 1.0);
    (square * (1.0 - out * out * (3.0 - 2.0 * out))) as f32
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
/// hands the shader the warp lens: the same stars crowd ahead, streak by their own apparent
/// motion and come back to rest, and the dust shows only while a warp inside a galaxy has some in.
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
                flow: Vec4::new(lens.phase, lens.flow, lens.medium(vantage.band), 0.0),
                dome: vantage.up.extend(skylight::luminance(palette.zenith)),
                base: Vec4::new(skylight::luminance(palette.horizon), 0.0, 0.0, 0.0),
            };
        }
    }
    let shown =
        [Visibility::Hidden, Visibility::Visible][usize::from(lens.medium(vantage.band) > 0.0)];
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
