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

use super::tune::LEVELS;

/// Colour banding of a camera. It runs on the camera's own low-resolution target, so it shades
/// one fragment per game pixel and its dither stays on the game pixels whatever the window size.
#[derive(Component, ExtractComponent, Clone, Copy, Debug, ShaderType)]
#[extract_app(RenderApp)]
pub struct Pixelate {
    /// Number of color levels per channel.
    pub levels: f32,
}

impl Default for Pixelate {
    /// Returns the banding of the fixed look, [`LEVELS`] levels per channel.
    fn default() -> Self {
        Self { levels: LEVELS }
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
