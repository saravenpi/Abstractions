//! Fullscreen post-process that snaps the 3D view to a coarse pixel grid and bands its colors.

use bevy::asset::embedded_asset;
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::core_pipeline::{Core3dSystems, schedule::Core3d};
use bevy::ecs::schedule::{IntoScheduleConfigs, ScheduleConfigs, ScheduleLabel};
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use bevy::render::{RenderApp, extract_component::ExtractComponent, render_resource::ShaderType};
use bevy::shader::ShaderRef;

/// Color levels per channel kept by the retro banding.
pub const COLOR_LEVELS: f32 = 24.0;

/// Pixelation settings of a camera, read by the post-process shader.
#[derive(Component, ExtractComponent, Clone, Copy, Debug, ShaderType)]
#[extract_app(RenderApp)]
pub struct Pixelate {
    /// Width and height of one block, in window pixels.
    pub pixel_size: f32,
    /// Number of color levels per channel.
    pub levels: f32,
}

impl Pixelate {
    /// Builds the effect for a block size taken from the settings.
    pub fn new(pixel_size: u32) -> Self {
        Self {
            pixel_size: pixel_size.max(1) as f32,
            levels: COLOR_LEVELS,
        }
    }
}

impl Default for Pixelate {
    /// Returns a 3 pixel block with the default color levels.
    fn default() -> Self {
        Self::new(3)
    }
}

impl FullscreenMaterial for Pixelate {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/pixelate.wgsl".into()
    }

    /// Runs the effect in the 3D schedule.
    fn schedule() -> impl ScheduleLabel + Clone {
        Core3d
    }

    /// Runs after tonemapping so banding happens on display colors.
    fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
        system.in_set(Core3dSystems::PostProcess).after(tonemapping)
    }
}

/// Embeds the shader and registers the post-process.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "pixelate.wgsl");
    app.add_plugins(FullscreenMaterialPlugin::<Pixelate>::default());
}
