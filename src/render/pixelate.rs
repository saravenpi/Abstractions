//! Fullscreen post-process that bands the colors of the low-resolution 3D view, with a dither of
//! interleaved gradient noise that softens toward the true colour on bright surfaces.

use bevy::asset::embedded_asset;
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::core_pipeline::{Core3dSystems, schedule::Core3d};
use bevy::ecs::schedule::{IntoScheduleConfigs, ScheduleConfigs, ScheduleLabel};
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use bevy::render::{RenderApp, extract_component::ExtractComponent, render_resource::ShaderType};
use bevy::shader::ShaderRef;

/// Color levels per channel used when no setting says otherwise.
const DEFAULT_LEVELS: u32 = 24;

/// Pixelation settings of a camera: the block size sizes its render target, the levels band its colors.
#[derive(Component, ExtractComponent, Clone, Copy, Debug, ShaderType)]
#[extract_app(RenderApp)]
pub struct Pixelate {
    /// Width and height of one block, in window pixels.
    pub pixel_size: u32,
    /// Number of color levels per channel.
    pub levels: f32,
}

impl Pixelate {
    /// Builds the effect for a block size and color level count taken from the settings.
    pub fn new(pixel_size: u32, levels: u32) -> Self {
        Self {
            pixel_size: pixel_size.max(1),
            levels: levels.max(2) as f32,
        }
    }
}

impl Default for Pixelate {
    /// Returns a 3 pixel block with the default color levels.
    fn default() -> Self {
        Self::new(3, DEFAULT_LEVELS)
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
