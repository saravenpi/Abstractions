//! How much work Solari gets: the full settings on a GPU with ray tracing hardware, lighter ones without it.

use bevy::prelude::*;
use bevy::solari::prelude::SolariLighting;

use super::gpu::Probe;

/// Sampler array elements wgpu reports from the Apple9 family on, the first with ray tracing hardware.
const APPLE9_SAMPLER_ELEMENTS: u32 = 500_000;

/// The Solari workload chosen for the device.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tier {
    /// The settings the camera asks for.
    #[default]
    Full,
    /// Fewer bounces, light samples and cache updates, for a GPU that traces rays in plain compute.
    Light,
}

impl Tier {
    /// Returns the tier as a phrase for the diagnostics file.
    pub fn words(self) -> &'static str {
        match self {
            Tier::Full => "full",
            Tier::Light => "light (no ray tracing hardware)",
        }
    }
}

/// Chooses the tier for a device: light on an Apple GPU older than the Apple9 family (M1, M2).
///
/// wgpu does not expose the GPU family, but its sampler array limit differs: 1024 before Apple9, 500000 from it.
pub fn choose(probe: &Probe) -> Tier {
    match probe.apple && probe.sampler_elements < APPLE9_SAMPLER_ELEMENTS {
        true => Tier::Light,
        false => Tier::Full,
    }
}

/// Lowers the cost of `lighting` to the light tier and keeps everything else as the camera set it.
///
/// A pixel then costs at most four rays for its path instead of six, and the world cache updates a quarter
/// of the cells per frame with a quarter of the light candidates.
pub fn lighten(lighting: &mut SolariLighting) {
    lighting.max_bounces = 2;
    lighting.primary_di_samples = 4;
    lighting.secondary_di_samples = 2;
    lighting.world_cache_direct_light_sample_count = 8;
    lighting.world_cache_cell_updates_soft_target = 10_000;
}

/// Applies the light tier to every camera that just got Solari lighting.
fn tune(tier: Res<Tier>, mut cameras: Query<&mut SolariLighting, Added<SolariLighting>>) {
    if *tier == Tier::Light {
        cameras
            .iter_mut()
            .for_each(|mut lighting| lighten(&mut lighting));
    }
}

/// Registers the tier and the system that applies it.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Tier>().add_systems(Update, tune);
}

#[cfg(test)]
mod tests;
