//! Ray traced lighting through Solari: the startup decision, GPU check and settings hint.

mod boot;

use bevy::pbr::DefaultOpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::renderer::RenderDevice;
use bevy::render::settings::{WgpuSettings, WgpuSettingsPriority};
use bevy::solari::SolariPlugins;

use boot::Start;

use crate::settings::{Persistent, Settings, config_dir};

/// Whether ray traced lighting is really active in this run.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RayTracing(pub bool);

/// What the running game knows about ray tracing since startup.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RayTracingBoot {
    /// Whether the game started with the Solari plugins.
    pub requested: bool,
    /// Whether the GPU offers every feature Solari needs.
    pub supported: bool,
}

/// Run condition: true when ray traced lighting is active.
pub fn active(ray_tracing: Res<RayTracing>) -> bool {
    ray_tracing.0
}

/// Returns the line the settings menu shows under the list, if any.
pub fn hint(wanted: bool, boot: RayTracingBoot) -> Option<&'static str> {
    if wanted && !boot.supported {
        Some("Ray tracing is not supported on this GPU")
    } else if wanted != boot.requested {
        Some("Restart the game to apply ray tracing")
    } else {
        None
    }
}

/// Runs the boot guard on the loaded settings and returns whether ray tracing should start.
///
/// A start that never finished turns the setting off and saves it.
pub fn begin(settings: &mut Persistent<Settings>) -> bool {
    let start = boot::begin(settings.graphics.ray_tracing, &config_dir());
    if start == Start::Recovered
        && let Err(error) = settings.update(|s| s.graphics.ray_tracing = false)
    {
        warn!("could not save settings: {error}");
    }
    start == Start::On
}

/// Returns the render plugin that asks wgpu for every feature the adapter has, Solari's among them.
///
/// `features` is left alone: forcing a feature the adapter lacks makes device creation panic.
pub fn render_plugin() -> RenderPlugin {
    RenderPlugin {
        render_creation: WgpuSettings {
            priority: WgpuSettingsPriority::Functionality,
            ..default()
        }
        .into(),
        ..default()
    }
}

/// Adds the Solari plugins and the boot marker systems. Call after `DefaultPlugins`.
pub fn enable(app: &mut App) {
    app.add_plugins(SolariPlugins)
        .insert_resource(RayTracingBoot {
            requested: true,
            supported: false,
        })
        .insert_resource(boot::BootMarker(config_dir().join(boot::MARKER)));
}

/// Checks the render device once: ray tracing is active only when Solari was added and the GPU supports it.
fn verify(
    device: Option<Res<RenderDevice>>,
    mut boot: ResMut<RayTracingBoot>,
    mut ray_tracing: ResMut<RayTracing>,
    method: Option<ResMut<DefaultOpaqueRendererMethod>>,
) {
    let needed = SolariPlugins::required_wgpu_features();
    boot.supported = device.is_some_and(|device| device.features().contains(needed));
    ray_tracing.0 = boot.requested && boot.supported;
    if boot.requested && !boot.supported {
        warn!("ray tracing needs GPU features this adapter lacks, so the game runs without it");
        if let Some(mut method) = method {
            method.set_to_forward();
        }
    }
}

/// Registers the resources, the GPU check and the boot marker systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RayTracing>()
        .init_resource::<RayTracingBoot>()
        .add_plugins(boot::plugin)
        .add_systems(Startup, verify);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hint covers unsupported GPUs and settings that differ from the running game.
    #[test]
    fn hint_follows_setting_and_boot() {
        let running = RayTracingBoot {
            requested: true,
            supported: true,
        };
        let plain = RayTracingBoot {
            requested: false,
            supported: true,
        };
        let lacking = RayTracingBoot {
            requested: false,
            supported: false,
        };
        assert_eq!(hint(true, running), None);
        assert_eq!(hint(false, plain), None);
        assert!(hint(false, running).unwrap().starts_with("Restart"));
        assert!(hint(true, plain).unwrap().starts_with("Restart"));
        assert!(hint(true, lacking).unwrap().contains("not supported"));
        assert_eq!(hint(false, lacking), None);
    }
}
