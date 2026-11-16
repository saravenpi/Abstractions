//! The live switch: GPU check at startup, the active state each frame and the default render method.

use bevy::ecs::system::SystemParam;
use bevy::pbr::DefaultOpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice};

use super::boot;
use super::gpu::{Probe, Support, check};
use super::{RayTracing, RayTracingBoot};
use crate::settings::{Persistent, Settings, config_dir};

/// The render device, adapter and adapter info, when the renderer exists.
#[derive(SystemParam)]
pub struct Gpu<'w> {
    /// The render device.
    pub device: Option<Res<'w, RenderDevice>>,
    /// The render adapter.
    pub adapter: Option<Res<'w, RenderAdapter>>,
    /// Name, backend and type of the adapter.
    pub info: Option<Res<'w, RenderAdapterInfo>>,
}

impl Gpu<'_> {
    /// Reads what the device and adapter offer.
    pub fn probe(&self) -> Option<Probe> {
        let (device, adapter) = self.device.as_deref().zip(self.adapter.as_deref())?;
        Some(Probe::of(device, adapter))
    }
}

/// Checks the device once and puts the default opaque method back to forward.
///
/// The Solari plugin sets the default to deferred when it is built, even on a GPU that cannot run it.
fn verify(
    gpu: Gpu,
    mut boot: ResMut<RayTracingBoot>,
    method: Option<ResMut<DefaultOpaqueRendererMethod>>,
) {
    boot.support = gpu.probe().map_or(Support::Unchecked, |p| check(&p));
    if let Some(mut method) = method {
        method.set_to_forward();
    }
}

/// Sets the active state from the setting and what the start allows; arms the boot guard when it switches on.
fn decide(
    mut commands: Commands,
    settings: Res<Persistent<Settings>>,
    boot: Res<RayTracingBoot>,
    mut ray_tracing: ResMut<RayTracing>,
) {
    let wanted = settings.graphics.ray_tracing && boot.ready();
    if ray_tracing.0 == wanted {
        return;
    }
    ray_tracing.0 = wanted;
    if wanted {
        boot::rearm(&mut commands, &config_dir());
    }
}

/// Follows the active state with the default opaque method: deferred for Solari, forward otherwise.
fn sync_method(ray_tracing: Res<RayTracing>, method: Option<ResMut<DefaultOpaqueRendererMethod>>) {
    let Some(mut method) = method else {
        return;
    };
    match ray_tracing.0 {
        true => method.set_to_deferred(),
        false => method.set_to_forward(),
    }
}

/// Registers the GPU check, the decision and the method switch.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, verify)
        .add_systems(PreUpdate, decide)
        .add_systems(Update, sync_method.run_if(resource_changed::<RayTracing>));
}
