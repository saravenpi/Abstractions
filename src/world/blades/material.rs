//! The material of the blades and the look of the turf both the blades and the ground read: the colours,
//! height, density and stiffness of the ground cover of the planet, and the wind that bends the blades.

use bevy::asset::RenderAssetUsages;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;

use crate::world::scatter::Turf;

/// The material of the blades: the standard lit material, its vertex shader growing each blade.
pub type BladeMaterial = ExtendedMaterial<StandardMaterial, Blades>;

/// The look of the turf as the shaders read it: the `Turf` block of `turf.wesl`.
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct TurfUniform {
    /// Linear RGB of the commonest ground-cover species, and 1 in `w` on a planet with turf.
    pub low: Vec4,
    /// Linear RGB of the next one, and how stiffly its blades stand in `w`, 0 to 1.
    pub high: Vec4,
    /// Height of a grown blade in metres, the density of the carpet from 0 to 1, then unused.
    pub shape: Vec4,
}

impl TurfUniform {
    /// Returns the block of `turf`, or the block of bare ground without one.
    pub fn of(turf: Option<&Turf>) -> Self {
        let Some(turf) = turf else {
            return Self::default();
        };
        let [low, high] = turf.colours.map(Vec3::from_array);
        Self {
            low: low.extend(1.0),
            high: high.extend(turf.stiffness),
            shape: Vec4::new(turf.height, turf.density, 0.0, 0.0),
        }
    }
}

/// What the blade shader adds to the standard material.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct Blades {
    /// The light of the planet under the camera, shared with the ground material.
    #[texture(100, filterable = false)]
    pub place: Handle<Image>,
    /// The look of the turf.
    #[uniform(101)]
    pub turf: TurfUniform,
    /// The wind in one texel: its velocity in metres per second in render axes, and its gust strength.
    /// It is a texture so that it can change every frame without the renderer taking up every blade
    /// mesh again, as a changed material would make it.
    #[texture(102, filterable = false)]
    pub wind: Handle<Image>,
}

impl MaterialExtension for Blades {
    /// Points at the shader that grows each blade from its root.
    fn vertex_shader() -> ShaderRef {
        "embedded://abstractions/world/blades/blades.wesl".into()
    }

    /// Points at the shader that lights the blades as the ground is lit.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/world/blades/blades.wesl".into()
    }

    /// The blades stay out of the depth prepass, whose vertex shader would not grow them.
    fn enable_prepass() -> bool {
        false
    }

    /// The blades cast no shadow.
    fn enable_shadows() -> bool {
        false
    }
}

/// Returns the one-texel wind texture, still air.
pub fn wind_map() -> Image {
    let size = Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    Image::new(
        size,
        TextureDimension::D2,
        vec![0; 16],
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    )
}

/// Returns the bytes of the wind texel for a wind of `velocity` metres per second and gust `strength`.
pub fn wind_texel(velocity: Vec3, strength: f32) -> [u8; 16] {
    let mut bytes = [0; 16];
    let numbers = velocity.extend(strength).to_array();
    for (slot, number) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(numbers) {
        *slot = number.to_le_bytes();
    }
    bytes
}

/// Returns the blade material over the light texture `place` and the wind texture `wind`.
pub fn blade_material(place: Handle<Image>, wind: Handle<Image>) -> BladeMaterial {
    let base = StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.95,
        reflectance: 0.05,
        double_sided: true,
        cull_mode: None,
        ..default()
    };
    let extension = Blades {
        place,
        turf: TurfUniform::default(),
        wind,
    };
    ExtendedMaterial { base, extension }
}
