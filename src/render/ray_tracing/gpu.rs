//! What the GPU must offer for Solari, checked the same way when the plugins finish and at startup.

use bevy::render::render_resource::{TextureFormat, TextureFormatFeatureFlags};
use bevy::render::renderer::{RenderAdapter, RenderDevice};
use bevy::render::settings::WgpuFeatures;

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

/// Binding array elements per shader stage Solari asks for: 5000 textures and 500 vertex and 500 index buffers.
pub const BINDING_ELEMENTS: u32 = 6_000;

/// Sampler binding array elements per shader stage Solari asks for: one per texture.
pub const SAMPLER_ELEMENTS: u32 = 5_000;

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
        Self {
            features: device.features(),
            binding_elements: limits.max_binding_array_elements_per_shader_stage,
            sampler_elements: limits.max_binding_array_sampler_elements_per_shader_stage,
            hdr_storage: format
                .flags
                .contains(TextureFormatFeatureFlags::STORAGE_READ_WRITE),
        }
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
    let roomy = probe.binding_elements >= BINDING_ELEMENTS
        && probe.sampler_elements >= SAMPLER_ELEMENTS
        && probe.hdr_storage;
    match roomy {
        true => Support::Ready,
        false => Support::TooLimited,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::solari::SolariPlugins;

    /// The names cover exactly the features Solari asks for.
    #[test]
    fn features_match_solari() {
        let all = FEATURES
            .iter()
            .fold(WgpuFeatures::empty(), |all, (_, flag)| all | *flag);
        assert_eq!(all, SolariPlugins::required_wgpu_features());
    }

    /// Missing features beat small limits, and small limits beat ready.
    #[test]
    fn check_names_the_blocker() {
        let ready = Probe {
            features: SolariPlugins::required_wgpu_features(),
            binding_elements: 1_000_000,
            sampler_elements: 500_000,
            hdr_storage: true,
        };
        let small = Probe {
            sampler_elements: 1024,
            ..ready
        };
        let blind = Probe {
            features: WgpuFeatures::empty(),
            ..small
        };
        assert_eq!(check(&ready), Support::Ready);
        assert_eq!(check(&small), Support::TooLimited);
        assert_eq!(check(&blind), Support::NoHardware);
        assert_eq!(missing(blind.features).len(), FEATURES.len());
    }
}
