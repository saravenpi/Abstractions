//! The starfield: the real stars near the camera and the far galaxies, as one mesh on a far shell.

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

pub use specks::{Speck, galaxy_speck, mesh, star_speck};

use super::palette::Palette;
use super::vantage::{Scene, Vantage};
use crate::screens::Screen;
use crate::space::UniPos;
use crate::universe::{GLY, GalaxyId, SECTOR_GLY, Universe};

/// Radius of the star shell in metres: behind every body, in front of the dome.
pub const STAR_SHELL: f32 = 3.0e6;

/// Distance in game light-years up to which stars are drawn.
const REACH_GLY: f64 = 40.0;

/// Most stars drawn, the nearest ones.
const MOST_STARS: usize = 4000;

/// Galaxy grid cells searched for galaxies around the camera's own.
const GALAXY_CELLS: i32 = 2;

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

/// Returns every speck seen from `at` metres from the centre of `galaxy`: the stars within reach
/// except the one of the current system, which is drawn as a body, and the galaxies around.
pub fn gather(universe: &Universe, galaxy: GalaxyId, at: DVec3, own: DVec3) -> Vec<Speck> {
    let stars = universe.stars_near(galaxy, at, REACH_GLY);
    let away = |pos: DVec3| pos.distance_squared(own) > 1.0;
    let stars = stars.iter().filter(|star| away(star.pos));
    let mut specks: Vec<Speck> = stars
        .take(MOST_STARS)
        .map(|star| star_speck(star, at))
        .collect();
    let Some(here) = universe.galaxy(galaxy).map(|spec| spec.locate(at)) else {
        return specks;
    };
    let others = universe.galaxies_near(galaxy, GALAXY_CELLS);
    let seen = |other: &_| galaxy_speck(other, UniPos::new(other.id.0, other.center) - here);
    specks.extend(others.iter().filter_map(seen));
    specks
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

/// Rebuilds the mesh when the camera has moved more than a sector since the last build.
///
/// Waits for the star system of the world to be known, so a stale place is never built for.
fn refresh(
    mut commands: Commands,
    scene: Scene,
    universe: Option<Res<Universe>>,
    mut fields: Query<(Entity, &mut Starfield, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (Some(universe), Some(_)) = (universe, scene.local.0.as_ref()) else {
        return;
    };
    let vantage = &scene.vantage;
    let (galaxy, at) = (vantage.galaxy, vantage.galactic);
    let near =
        |(built, was): (GalaxyId, DVec3)| built == galaxy && was.distance(at) < SECTOR_GLY * GLY;
    for (entity, mut field, mut visibility) in &mut fields {
        if field.built.is_some_and(near) {
            continue;
        }
        field.built = Some((galaxy, at));
        let own = vantage.galactic - vantage.observer.eye;
        let specks = gather(&universe, galaxy, at, own);
        if specks.is_empty() {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        commands
            .entity(entity)
            .try_insert(Mesh3d(meshes.add(mesh(&specks))));
    }
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
        .add_systems(PostUpdate, (refresh, follow).after(super::sun::track));
}
