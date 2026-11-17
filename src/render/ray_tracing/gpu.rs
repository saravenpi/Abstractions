//! What the GPU must offer for Solari, checked the same way when the plugins finish and at startup.

use bevy::render::render_resource::{TextureFormat, TextureFormatFeatureFlags};
use bevy::render::renderer::{RenderAdapter, RenderDevice};
use bevy::render::settings::{Backends, WgpuFeatures};
use bevy::solari::scene::texture_slot_capacity;

use super::env;

/// Every wgpu feature Solari needs, by name.
pub const FEATURES: [(&str, WgpuFeatures); 5] = [
    (
        "EXPERIMENTAL_RAY_QUERY",
        WgpuFeatures::EXPERIMENTAL_RAY_QUERY,
    ),
    ("BUFFER_BINDING_ARRAY", WgpuFeatures::BUFFER_BINDING_ARRAY),
    ("TEXTURE_BINDING_ARRAY", WgpuFeatures::TEXTURE_BINDING_ARRAY),
    (
        "SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING",
        WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
    ),
    (
        "PARTIALLY_BOUND_BINDING_ARRAY",
        WgpuFeatures::PARTIALLY_BOUND_BINDING_ARRAY,
    ),
];

/// Mesh slab buffers Solari binds in arrays: 500 vertex and 500 index buffers.
pub const SLAB_ELEMENTS: u32 = 1_000;

/// Texture and sampler slots the game needs in Solari's arrays: it binds two textures, the rest is headroom.
pub const TEXTURE_SLOTS: u32 = 16;

/// The numbers read from the device and adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Probe {
    /// Features the device was created with.
    pub features: WgpuFeatures,
    /// Binding array elements per shader stage the device allows.
    pub binding_elements: u32,
    /// Sampler binding array elements per shader stage the device allows.
    pub sampler_elements: u32,
    /// Whether the HDR target format can be bound for reading and writing.
    pub hdr_storage: bool,
    /// Whether the device is an Apple GPU on Metal, or the debug profile that imitates one.
    pub apple: bool,
}

/// What the check found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Support {
    /// Nothing was checked yet.
    #[default]
    Unchecked,
    /// The device offers everything Solari needs.
    Ready,
    /// A required wgpu feature is missing, which means no hardware ray tracing.
    NoHardware,
    /// The features exist but a device limit is below what Solari binds.
    TooLimited,
}

impl Probe {
    /// Reads the probe from the render device and adapter.
    pub fn of(device: &RenderDevice, adapter: &RenderAdapter) -> Self {
        let limits = device.limits();
        let format = adapter.get_texture_format_features(TextureFormat::Rgba16Float);
        let metal = Backends::from(adapter.get_info().backend).contains(Backends::METAL);
        Self {
            features: device.features(),
            binding_elements: limits.max_binding_array_elements_per_shader_stage,
            sampler_elements: limits.max_binding_array_sampler_elements_per_shader_stage,
            hdr_storage: format
                .flags
                .contains(TextureFormatFeatureFlags::STORAGE_READ_WRITE),
            apple: metal || env::apple8(),
        }
    }

    /// Returns the length of the texture and sampler arrays Solari binds on this device.
    pub fn texture_slots(&self) -> u32 {
        texture_slot_capacity(self.sampler_elements)
    }

    /// Returns the binding array elements per shader stage Solari binds on this device.
    pub fn elements_needed(&self) -> u32 {
        SLAB_ELEMENTS + 2 * self.texture_slots()
    }
}

/// Returns the names of the required features that `features` lacks.
pub fn missing(features: WgpuFeatures) -> Vec<&'static str> {
    FEATURES
        .iter()
        .filter(|(_, flag)| !features.contains(*flag))
        .map(|(name, _)| *name)
        .collect()
}

/// Judges a probe: missing features mean no hardware, small limits mean too limited.
pub fn check(probe: &Probe) -> Support {
    if !missing(probe.features).is_empty() {
        return Support::NoHardware;
    }
    let roomy = probe.texture_slots() >= TEXTURE_SLOTS
        && probe.binding_elements >= probe.elements_needed()
        && probe.hdr_storage;
    match roomy {
        true => Support::Ready,
        false => Support::TooLimited,
    }
}

#[cfg(test)]
mod tests;
