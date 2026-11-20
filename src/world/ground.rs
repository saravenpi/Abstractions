//! The material of the ground: the standard lit material, with the light of the place it shows.
//!
//! The lights of the scene are the lights at the camera. Ground seen from far stands under
//! another height of the star, so its shader swaps the light of the camera for the light of its
//! own place, by the rule of `render::daylight`. Under the camera the two are the same and the
//! ground is lit exactly as the standard material lights it.

use bevy::asset::{AssetEventSystems, RenderAssetUsages, embedded_asset};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;

mod season;

use super::blades::TurfUniform;
use super::props::PropAssets;
use crate::render::{PlaceLight, PlaceLit, PlaceUniform, SeasonLight, SeasonLit};

pub use season::season_map;

/// The material every mesh of the world shares: terrain near and far, props, grass and buildings.
pub type GroundMaterial = ExtendedMaterial<StandardMaterial, Daylit>;

/// What the ground shader adds to the standard material: the light of the planet under the camera.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct Daylit {
    /// The star, the air and the planet the ground is lit by, and the light the scene carries:
    /// the block of [`PlaceUniform`], one field per texel. It is a texture and not a uniform
    /// because it changes every frame, and a texture is rewritten in place, while a changed
    /// material makes the renderer take up again every mesh that is drawn with it.
    #[texture(100, filterable = false)]
    pub place: Handle<Image>,
    /// The look of the turf of the planet, which carpets the open ground of its living planets.
    #[uniform(101)]
    pub turf: TurfUniform,
    /// The season of the planet under the camera: the block of `SeasonUniform`, one field per texel, in
    /// a texture for the same reason as the light of the place.
    #[texture(102, filterable = false)]
    pub season: Handle<Image>,
}

impl MaterialExtension for Daylit {
    /// Points at the fragment shader embedded in the binary. The depth and shadow passes keep
    /// the shaders of the standard material.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/world/ground.wesl".into()
    }
}

/// Returns the bytes of the texels that hold `place`, in the order `place_from` of
/// `daylight.wesl` reads them.
pub fn texels(place: &PlaceUniform) -> [u8; PlaceUniform::BYTES] {
    let mut bytes = [0; PlaceUniform::BYTES];
    let numbers = place
        .fields()
        .into_iter()
        .flat_map(|field| field.to_array());
    for (slot, number) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(numbers) {
        *slot = number.to_le_bytes();
    }
    bytes
}

/// Builds the texture that carries the light block to the ground shader, all zero: no light of
/// a place, so the ground is lit as the standard material lights it.
pub fn place_map() -> Image {
    let size = Extent3d {
        width: PlaceUniform::FIELDS as u32,
        height: 1,
        depth_or_array_layers: 1,
    };
    Image::new(
        size,
        TextureDimension::D2,
        vec![0; PlaceUniform::BYTES],
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    )
}

/// Writes the light of the planet under the camera into the texture of the ground material,
/// in place, whenever it changed.
pub fn light(
    place: Res<PlaceLight>,
    assets: Option<Res<PropAssets>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(assets) = assets else {
        return;
    };
    let bytes = texels(&place.0);
    let same = |image: &Image| image.data.as_deref() == Some(&bytes[..]);
    if images.get(&assets.place).is_none_or(same) {
        return;
    }
    let Some(mut image) = images.get_mut(&assets.place) else {
        return;
    };
    if let Some(data) = image.data.as_mut() {
        data.copy_from_slice(&bytes);
    }
}

/// Embeds the shader, registers the material and its warm-up, and lights it once the camera is
/// tracked, before the frame's asset changes are sent to the renderer.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "ground.wesl");
    if app
        .world()
        .contains_resource::<Assets<bevy::shader::Shader>>()
    {
        bevy::shader::load_shader_library!(app, "turf.wesl");
    }
    app.add_plugins(MaterialPlugin::<GroundMaterial>::default())
        .init_resource::<PlaceLight>()
        .init_resource::<SeasonLight>()
        .add_systems(
            PostUpdate,
            (
                light.after(PlaceLit),
                season::follow
                    .after(SeasonLit)
                    .run_if(resource_exists::<PropAssets>),
            )
                .before(AssetEventSystems),
        );
    crate::render::warm_material::<GroundMaterial>(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the numbers the texture of the ground material of `app` holds, and whether the
    /// material still is the standard one with its lookup and its finish.
    fn ground(app: &App) -> (Vec<f32>, bool) {
        let assets = app.world().resource::<PropAssets>();
        let materials = app.world().resource::<Assets<GroundMaterial>>();
        let images = app.world().resource::<Assets<Image>>();
        let material = materials
            .get(&assets.material)
            .expect("the ground material");
        let map = images.get(&material.extension.place).expect("its light");
        let bytes = map.data.as_deref().unwrap_or_default().as_chunks::<4>().0;
        let base = &material.base;
        let kept = base.base_color_texture.is_some()
            && (base.perceptual_roughness, base.reflectance) == (0.95, 0.05);
        (bytes.iter().map(|b| f32::from_le_bytes(*b)).collect(), kept)
    }

    /// The ground starts with no light of a place, so it is the standard material, and its
    /// texture takes the light of the planet as soon as there is one, field by field in the
    /// order of the block, with the lookup and the finish of the material kept.
    #[test]
    fn the_ground_takes_the_light_of_the_planet() {
        let mut app = App::new();
        let mut materials = Assets::<GroundMaterial>::default();
        let mut images = Assets::<Image>::default();
        app.insert_resource(PropAssets::new(&mut materials, &mut images))
            .insert_resource(materials)
            .insert_resource(images)
            .init_resource::<PlaceLight>()
            .add_systems(Update, light);
        app.update();
        assert_eq!(ground(&app), (vec![0.0; 4 * PlaceUniform::FIELDS], true));
        let lit = PlaceUniform {
            centre: Vec4::new(0.0, -6000.0, 0.0, 6000.0),
            air: Vec4::new(4000.0, 60.0, 1.0, 0.0),
            eye_sky: Vec4::new(1.0, 2.0, 3.0, 4.0),
            ..default()
        };
        app.world_mut().resource_mut::<PlaceLight>().0 = lit;
        app.update();
        let (numbers, kept) = ground(&app);
        let fields: Vec<f32> = lit.fields().iter().flat_map(Vec4::to_array).collect();
        assert!(kept && numbers == fields);
        assert_eq!(numbers[12..16], [4000.0, 60.0, 1.0, 0.0]);
        assert_eq!(numbers[24..], [1.0, 2.0, 3.0, 4.0]);
    }
}
