//! Ray traced lighting through Solari: the start decision, the live toggle, diagnostics and the status line.

mod apply;
mod boot;
mod env;
mod gpu;
mod guard;
mod report;
mod sky;
mod status;
mod tier;
mod trace;

use bevy::prelude::*;
use bevy::solari::realtime::SolariLightingPlugin;
use bevy::solari::scene::RaytracingScenePlugin;

use self::boot::BootMarker;
pub use self::boot::Start;
pub use self::env::render_plugin;
pub use self::gpu::Support;
use self::guard::Guarded;
pub use self::status::status;
use crate::settings::{Persistent, Settings, config_dir};

/// Whether ray traced lighting is really active right now.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RayTracing(pub bool);

/// What the start decided about the Solari plugins and what the GPU check found.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RayTracingBoot {
    /// What the boot guard decided; the plugins run only when it is armed.
    pub guard: Start,
    /// The result of the GPU check.
    pub support: Support,
}

impl RayTracingBoot {
    /// Returns whether the plugins were added and the GPU can run them.
    pub fn ready(&self) -> bool {
        self.guard == Start::Armed && self.support == Support::Ready
    }
}

/// Run condition: true when ray traced lighting is active.
pub fn active(ray_tracing: Res<RayTracing>) -> bool {
    ray_tracing.0
}

/// Runs the boot guard and returns what it decided.
///
/// A start that never finished turns the setting off and saves it. The `ABSTRACTIONS_RAY_TRACING=off`
/// environment switch skips the plugins and leaves the setting alone.
pub fn begin(settings: &mut Persistent<Settings>) -> Start {
    if env::off() {
        return Start::Disabled;
    }
    let start = boot::begin(&config_dir());
    if start == Start::Recovered
        && let Err(error) = settings.update(|s| s.graphics.ray_tracing = false)
    {
        warn!("could not save settings: {error}");
    }
    start
}

/// Adds the Solari plugins when the guard allows it and records the decision. Call after `DefaultPlugins`.
///
/// The plugins are added whether or not the setting is on, so the setting can change while the game runs.
pub fn prepare(app: &mut App, start: Start) {
    app.insert_resource(RayTracingBoot {
        guard: start,
        support: Support::Unchecked,
    });
    if start == Start::Armed {
        app.add_plugins((
            Guarded(RaytracingScenePlugin),
            Guarded(SolariLightingPlugin),
        ))
        .insert_resource(BootMarker::default());
    }
}

/// Registers the resources, the GPU check, the live toggle and the diagnostics.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RayTracing>().add_plugins((
        boot::plugin,
        apply::plugin,
        sky::plugin,
        tier::plugin,
        trace::plugin,
    ));
}
