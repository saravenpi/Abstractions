//! The sky material: an unlit dome shader with a gradient that fades with height, the glow of the
//! sun, the horizon rim and the light of the galaxies, read from their baked cube map. Moons are
//! real bodies, drawn with the planets.

use bevy::asset::embedded_asset;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::celestial::smoothstep;
use super::palette::Palette;
use super::skylight;

/// Uniform block read by `sky.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct SkyUniform {
    /// Linear sky color straight up.
    pub zenith: Vec4,
    /// Linear sky color at the horizon.
    pub horizon: Vec4,
    /// Sun direction in xyz and the strength of its glow in the air in w.
    pub sun: Vec4,
    /// Linear cloud color in xyz and how much of the view the cloud whiteout takes in w.
    pub cloud: Vec4,
    /// Local up in xyz and the sine of the angle the horizon drops by in w.
    pub up: Vec4,
    /// Linear color of the bright rim along the horizon, already scaled by its strength.
    pub rim: Vec4,
    /// Strength of the light of the galaxies in x, in y 1 once their map is baked, and in z the
    /// strength of the glare of the star seen under no air, see `glare`.
    pub galaxy: Vec4,
    /// First column of the rotation from render axes to galaxy axes, the axes of the map.
    pub to_galaxy_x: Vec4,
    /// Second column of that rotation.
    pub to_galaxy_y: Vec4,
    /// Third column of that rotation.
    pub to_galaxy_z: Vec4,
    /// Linear color of the star light the air lets through, which colors the glow around the star.
    pub glow: Vec4,
    /// Unit vector of the direction of travel in xyz and, in w, the speed the sky is seen
    /// through in a warp as a share of the speed of light: the band crowds ahead with the stars.
    pub warp: Vec4,
}

/// What the dome is drawn from besides the palette, with directions in render axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkyLook {
    /// Unit vector away from the planet center.
    pub up: Vec3,
    /// Sine of the angle the horizon drops by at this height.
    pub dip: f32,
    /// Sine of the height of the star above the horizon.
    pub sun_height: f32,
    /// Strength of the glare of the star, see `glare::strength`: 0 in the shadow of the planet
    /// and under a whole sky.
    pub halo: f32,
    /// Share of the atmosphere left above the camera.
    pub air: f32,
    /// How deep the camera is in the cloud layer, from 0 to 1.
    pub whiteout: f32,
    /// sRGB color of the rim along the horizon, already scaled by its strength.
    pub rim: Vec3,
    /// Rotation from render axes to galaxy axes.
    pub to_galaxy: Mat3,
    /// Whether the map of the light of the galaxies is baked.
    pub mapped: bool,
    /// sRGB colour of the liquid around a camera below the surface of a sea.
    pub murk: Option<Vec3>,
    /// Unit vector of the direction of travel.
    pub heading: Vec3,
    /// Speed the sky is seen through in a warp, as a share of the speed of light.
    pub beta: f32,
}

/// Converts an sRGB triple to a linear color vector.
pub fn linear(v: Vec3) -> Vec4 {
    Color::srgb(v.x, v.y, v.z).to_linear().to_vec4()
}

impl SkyUniform {
    /// Builds the uniform for a palette and what the camera's place adds to it.
    ///
    /// Below the surface of a sea the dome is the one flat colour of the liquid, without sun,
    /// rim or band.
    pub fn new(palette: &Palette, look: &SkyLook) -> Self {
        let dry = if look.murk.is_some() { 0.0 } else { 1.0 };
        let (zenith, horizon) = match look.murk {
            Some(liquid) => (liquid, liquid),
            None => (palette.zenith, palette.horizon),
        };
        let glow = smoothstep(-0.15, 0.05, look.sun_height) * look.air * dry;
        Self {
            zenith: linear(zenith),
            horizon: linear(horizon),
            sun: palette.sun_dir.extend(glow),
            cloud: linear(palette.cloud).truncate().extend(look.whiteout * dry),
            up: look.up.extend(look.dip),
            rim: linear(look.rim) * dry,
            galaxy: Vec4::new(
                skylight::band_of(palette.zenith, palette.horizon) * dry,
                f32::from(u8::from(look.mapped)),
                look.halo * dry,
                0.0,
            ),
            to_galaxy_x: look.to_galaxy.x_axis.extend(0.0),
            to_galaxy_y: look.to_galaxy.y_axis.extend(0.0),
            to_galaxy_z: look.to_galaxy.z_axis.extend(0.0),
            glow: linear(palette.sun_color),
            warp: look.heading.extend(look.beta),
        }
    }
}

/// Material of the sky dome.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SkyMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub sky: SkyUniform,
    /// The light of the galaxies: the cube map `galaxy_light` bakes, in galaxy axes.
    #[texture(1, dimension = "cube")]
    #[sampler(2)]
    pub map: Option<Handle<Image>>,
}

impl Material for SkyMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/sky.wesl".into()
    }

    /// The dome stays out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }

    /// The dome never takes part in shadow rendering.
    fn enable_shadows() -> bool {
        false
    }

    /// Draws both faces so the dome shows from the inside whatever its winding.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// Embeds the shader, registers the material and the map of the light of the galaxies it reads.
pub(super) fn plugin(app: &mut App) {
    super::galaxy_light::plugin(app);
    embedded_asset!(app, "sky.wesl");
    app.add_plugins(MaterialPlugin::<SkyMaterial>::default());
}
