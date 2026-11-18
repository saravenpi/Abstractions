//! The material of the ground: the standard lit material, with the light of the place it shows.
//!
//! The lights of the scene are the lights at the camera. Ground seen from far stands under
//! another height of the star, so its shader swaps the light of the camera for the light of its
//! own place, by the rule of `render::daylight`. Under the camera the two are the same and the
//! ground is lit exactly as the standard material lights it.

use bevy::asset::{AssetEventSystems, embedded_asset};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

use super::props::PropAssets;
use crate::render::{PlaceLight, PlaceLit, PlaceUniform};

/// The material every mesh of the world shares: terrain near and far, props, grass and buildings.
pub type GroundMaterial = ExtendedMaterial<StandardMaterial, Daylit>;

/// What the ground shader adds to the standard material: the light of the planet under the camera.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct Daylit {
    /// The star, the air and the planet the ground is lit by, and the light the scene carries.
    #[uniform(100)]
    pub place: PlaceUniform,
}

impl MaterialExtension for Daylit {
    /// Points at the fragment shader embedded in the binary. The depth and shadow passes keep
    /// the shaders of the standard material.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/world/ground.wesl".into()
    }
}

/// Hands the ground material the light of the planet under the camera, whenever it changed.
pub fn light(
    place: Res<PlaceLight>,
    assets: Option<Res<PropAssets>>,
    mut materials: ResMut<Assets<GroundMaterial>>,
) {
    let Some(assets) = assets else {
        return;
    };
    let same = |material: &GroundMaterial| material.extension.place == place.0;
    if materials.get(&assets.material).is_none_or(same) {
        return;
    }
    if let Some(mut material) = materials.get_mut(&assets.material) {
        material.extension.place = place.0;
    }
}

/// Embeds the shader, registers the material and its warm-up, and lights it once the camera is
/// tracked, before the frame's asset changes are sent to the renderer.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "ground.wesl");
    app.add_plugins(MaterialPlugin::<GroundMaterial>::default())
        .init_resource::<PlaceLight>()
        .add_systems(PostUpdate, light.after(PlaceLit).before(AssetEventSystems));
    crate::render::warm_material::<GroundMaterial>(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the ground material of `app`.
    fn ground(app: &App) -> GroundMaterial {
        let handle = &app.world().resource::<PropAssets>().material;
        let materials = app.world().resource::<Assets<GroundMaterial>>();
        materials.get(handle).cloned().unwrap_or_default()
    }

    /// The ground material starts with no light of its own, so it is the standard material, and
    /// takes the light of the planet as soon as there is one, with its lookup and its finish kept.
    #[test]
    fn the_ground_takes_the_light_of_the_planet() {
        let mut app = App::new();
        let mut materials = Assets::<GroundMaterial>::default();
        let mut images = Assets::<Image>::default();
        app.insert_resource(PropAssets::new(&mut materials, &mut images))
            .insert_resource(materials)
            .init_resource::<PlaceLight>()
            .add_systems(Update, light);
        app.update();
        assert_eq!(ground(&app).extension.place, PlaceUniform::default());
        let lit = PlaceUniform {
            centre: Vec4::new(0.0, -6000.0, 0.0, 6000.0),
            air: Vec4::new(4000.0, 60.0, 1.0, 0.0),
            ..default()
        };
        app.world_mut().resource_mut::<PlaceLight>().0 = lit;
        app.update();
        let base = ground(&app).base;
        assert_eq!(ground(&app).extension.place, lit);
        assert!(base.base_color_texture.is_some());
        assert_eq!((base.perceptual_roughness, base.reflectance), (0.95, 0.05));
    }
}
