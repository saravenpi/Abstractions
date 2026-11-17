//! Environment switches: one keeps the Solari plugins out, one starts the renderer with the limits of an Apple M2.

use bevy::render::RenderPlugin;
use bevy::render::settings::{WgpuLimits, WgpuSettings};

/// Set to `off` to start without the Solari plugins, whatever the settings file says.
pub const SWITCH: &str = "ABSTRACTIONS_RAY_TRACING";

/// Set to `apple8` to clamp the device limits to those of an Apple M2, to test that case on any machine.
pub const PROFILE: &str = "ABSTRACTIONS_GPU_PROFILE";

/// Returns whether the value of [`SWITCH`] asks for no ray tracing.
pub fn is_off(value: Option<&str>) -> bool {
    value.is_some_and(|v| ["off", "0", "false", "no"].contains(&v.trim().to_lowercase().as_str()))
}

/// Returns whether the environment keeps the Solari plugins out.
pub fn off() -> bool {
    is_off(std::env::var(SWITCH).ok().as_deref())
}

/// Returns whether the environment asks for the Apple M2 limits.
pub fn apple8() -> bool {
    std::env::var(PROFILE).is_ok_and(|v| v.trim().eq_ignore_ascii_case("apple8"))
}

/// Returns the per-stage binding limits wgpu 30 reports for an Apple8 GPU on Metal; every other limit is left open.
///
/// The numbers come from `wgpu-hal` 30.0.2 `metal/adapter.rs`: 1024 sampler array elements, no array of
/// acceleration structures, 16 samplers, 96 sampled and 32 storage textures, and 29 buffers of each kind.
pub fn apple8_limits() -> WgpuLimits {
    WgpuLimits {
        max_binding_array_elements_per_shader_stage: 1_000_000,
        max_binding_array_sampler_elements_per_shader_stage: 1024,
        max_binding_array_acceleration_structure_elements_per_shader_stage: 0,
        max_samplers_per_shader_stage: 16,
        max_sampled_textures_per_shader_stage: 96,
        max_storage_textures_per_shader_stage: 32,
        max_storage_buffers_per_shader_stage: 29,
        max_uniform_buffers_per_shader_stage: 29,
        max_vertex_buffers: 29,
        max_dynamic_uniform_buffers_per_pipeline_layout: 29,
        max_dynamic_storage_buffers_per_pipeline_layout: 29,
        max_acceleration_structures_per_shader_stage: 29,
        max_buffers_and_acceleration_structures_per_shader_stage: 29,
        ..WgpuLimits::unlimited()
    }
}

/// Returns the render plugin: the default one, or one clamped to the Apple M2 limits when [`PROFILE`] asks.
pub fn render_plugin() -> RenderPlugin {
    if !apple8() {
        return RenderPlugin::default();
    }
    let settings = WgpuSettings {
        constrained_limits: Some(apple8_limits()),
        ..Default::default()
    };
    RenderPlugin {
        render_creation: settings.into(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the usual words for "no" switch ray tracing off.
    #[test]
    fn the_switch_reads_off_words() {
        for word in ["off", "OFF", " 0 ", "false", "no"] {
            assert!(is_off(Some(word)), "{word}");
        }
        for word in ["on", "1", "", "apple8"] {
            assert!(!is_off(Some(word)), "{word}");
        }
        assert!(!is_off(None));
    }

    /// Clamping a roomy device by the profile lowers the binding limits and nothing else.
    #[test]
    fn the_profile_only_lowers_binding_limits() {
        let roomy = WgpuLimits {
            max_binding_array_sampler_elements_per_shader_stage: 500_000,
            max_samplers_per_shader_stage: 4_000,
            ..WgpuLimits::default()
        };
        let clamped = roomy.clone().or_worse_values_from(&apple8_limits());
        assert_eq!(
            clamped.max_binding_array_sampler_elements_per_shader_stage,
            1024
        );
        assert_eq!(clamped.max_samplers_per_shader_stage, 16);
        assert_eq!(
            clamped.max_texture_dimension_2d,
            roomy.max_texture_dimension_2d
        );
        assert_eq!(clamped.max_buffer_size, roomy.max_buffer_size);
        assert_eq!(
            clamped.min_uniform_buffer_offset_alignment,
            roomy.min_uniform_buffer_offset_alignment
        );
    }
}
