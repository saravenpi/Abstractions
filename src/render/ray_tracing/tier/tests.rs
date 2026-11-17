//! Tests of the Solari workload: which devices get the light tier and what it changes.

use super::*;
use bevy::render::settings::WgpuFeatures;

/// Builds a probe with the given Apple flag and sampler array limit.
fn probe(apple: bool, sampler_elements: u32) -> Probe {
    Probe {
        features: WgpuFeatures::empty(),
        binding_elements: 1_000_000,
        sampler_elements,
        hdr_storage: true,
        apple,
    }
}

/// Only an Apple GPU below the Apple9 sampler limit gets the light tier.
#[test]
fn m1_and_m2_get_the_light_tier() {
    assert_eq!(choose(&probe(true, 1024)), Tier::Light);
    assert_eq!(choose(&probe(true, 996)), Tier::Light);
    assert_eq!(choose(&probe(true, 500_000)), Tier::Full);
    assert_eq!(choose(&probe(false, 1024)), Tier::Full);
}

/// The light tier never raises a cost and keeps the camera's own choices.
#[test]
fn the_light_tier_only_lowers_costs() {
    let full = SolariLighting {
        restir: true,
        confidence_weight_cap: 16.0,
        ..default()
    };
    let mut light = full.clone();
    lighten(&mut light);
    assert!(light.max_bounces < full.max_bounces && light.max_bounces >= 1);
    assert!(light.primary_di_samples < full.primary_di_samples);
    assert!(light.secondary_di_samples < full.secondary_di_samples);
    assert!(light.world_cache_cell_updates_soft_target < full.world_cache_cell_updates_soft_target);
    assert!(light.restir && light.confidence_weight_cap == 16.0);
}

/// A camera that gets Solari lighting is tuned on the next update, on the light tier only.
#[test]
fn new_cameras_follow_the_tier() {
    for (tier, bounces) in [(Tier::Light, 2), (Tier::Full, 3)] {
        let mut app = App::new();
        app.add_plugins(plugin).insert_resource(tier);
        let camera = app.world_mut().spawn(SolariLighting::default()).id();
        app.update();
        let lighting = app.world().get::<SolariLighting>(camera).unwrap();
        assert_eq!(lighting.max_bounces, bounces);
    }
}
