//! Writes `ray-tracing.log`: the start entry and one entry per toggle and per settled toggle.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::solari::prelude::{RaytracingMesh3d, SolariLighting};

use super::apply::Gpu;
use super::gpu::{FEATURES, Probe, Support, TEXTURE_SLOTS, missing};
use super::report::{StartInfo, ToggleInfo, append, start, toggle, write};
use super::tier::Tier;
use super::{RayTracing, RayTracingBoot, Start};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings, config_dir};

/// Frames a toggle gets to take effect before its settled entry is written.
const SETTLE_FRAMES: u32 = 180;

/// Frames left before the settled entry of the last toggle is written.
#[derive(Resource)]
struct Settle(u32);

/// What the toggle entries count.
#[derive(SystemParam)]
struct Census<'w, 's> {
    meshes: Query<'w, 's, (), With<RaytracingMesh3d>>,
    lit: Query<'w, 's, (), With<SolariLighting>>,
    cameras: Query<'w, 's, &'static Camera, With<GameCamera>>,
    time: Res<'w, Time>,
    ray_tracing: Res<'w, RayTracing>,
}

impl Census<'_, '_> {
    /// Counts the entities now under `label`.
    fn entry(&self, label: &'static str) -> ToggleInfo {
        ToggleInfo {
            at: self.time.elapsed_secs(),
            label,
            active: self.ray_tracing.0,
            meshes: self.meshes.iter().count(),
            cameras: self.lit.iter().count(),
            target: self.cameras.iter().find_map(Camera::physical_target_size),
        }
    }
}

/// Returns the guard state as a word.
fn guard_word(start: Start) -> &'static str {
    match start {
        Start::Armed => "armed",
        Start::Recovered => "recovered from a failed start, plugins skipped",
        Start::Unwritable => "marker not writable, plugins skipped",
        Start::Disabled => "turned off by ABSTRACTIONS_RAY_TRACING, plugins skipped",
    }
}

/// Returns the check result as a word.
fn support_word(support: Support) -> &'static str {
    match support {
        Support::Unchecked => "not checked",
        Support::Ready => "ready",
        Support::NoHardware => "required wgpu features missing",
        Support::TooLimited => "a device limit is below what Solari binds",
    }
}

/// Returns the binding array limits of the probe and the array length Solari binds, each next to what is needed.
fn limits(probe: Option<Probe>) -> Vec<(&'static str, u32, u32)> {
    let Some(probe) = probe else {
        return Vec::new();
    };
    vec![
        (
            "max_binding_array_elements_per_shader_stage",
            probe.binding_elements,
            probe.elements_needed(),
        ),
        (
            "max_binding_array_sampler_elements_per_shader_stage",
            probe.sampler_elements,
            TEXTURE_SLOTS,
        ),
        (
            "texture and sampler slots Solari binds",
            probe.texture_slots(),
            TEXTURE_SLOTS,
        ),
    ]
}

/// Writes the start entry once, on the first frame, when the active state is known.
fn write_start(
    gpu: Gpu,
    boot: Res<RayTracingBoot>,
    settings: Res<Persistent<Settings>>,
    ray_tracing: Res<RayTracing>,
    workload: Res<Tier>,
) {
    let probe = gpu.probe();
    let absent = missing(probe.map_or(Default::default(), |p| p.features));
    let adapter =
        |i: &RenderAdapterInfo| format!("{} ({:?}, {:?})", i.name, i.backend, i.device_type);
    let info = StartInfo {
        version: env!("CARGO_PKG_VERSION"),
        adapter: gpu.info.as_deref().map_or_else(|| "none".into(), adapter),
        features: FEATURES
            .iter()
            .map(|(name, _)| (*name, !absent.contains(name)))
            .collect(),
        limits: limits(probe),
        hdr_storage: probe.is_some_and(|p| p.hdr_storage),
        support: support_word(boot.support).to_string(),
        workload: workload.words(),
        added: boot.guard == Start::Armed,
        guard: guard_word(boot.guard).to_string(),
        setting: settings.graphics.ray_tracing,
        active: ray_tracing.0,
    };
    write(&config_dir(), &start(&info));
}

/// Appends an entry when the active state changes, then waits for the change to settle.
fn note_toggle(mut commands: Commands, census: Census, mut seen: Local<bool>) {
    if std::mem::replace(&mut *seen, true) {
        append(&config_dir(), &toggle(&census.entry("toggled")));
        commands.insert_resource(Settle(SETTLE_FRAMES));
    }
}

/// Appends the settled entry once the frames of the last toggle passed.
fn settle(mut commands: Commands, mut left: ResMut<Settle>, census: Census) {
    left.0 = left.0.saturating_sub(1);
    if left.0 == 0 {
        append(&config_dir(), &toggle(&census.entry("settled")));
        commands.remove_resource::<Settle>();
    }
}

/// Registers the log systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, write_start.run_if(run_once))
        .add_systems(
            Update,
            (
                note_toggle.run_if(resource_changed::<RayTracing>),
                settle.run_if(resource_exists::<Settle>),
            )
                .chain(),
        );
}
