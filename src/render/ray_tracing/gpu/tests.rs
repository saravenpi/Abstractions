//! Tests of the GPU check: the decision and the array sizes for a set of device limits.

use super::*;
use bevy::solari::SolariPlugins;

/// The names cover exactly the features Solari asks for.
#[test]
fn features_match_solari() {
    let all = FEATURES
        .iter()
        .fold(WgpuFeatures::empty(), |all, (_, flag)| all | *flag);
    assert_eq!(all, SolariPlugins::required_wgpu_features());
}

/// Missing features beat small limits, and small limits beat ready.
#[test]
fn check_names_the_blocker() {
    let ready = Probe {
        features: SolariPlugins::required_wgpu_features(),
        binding_elements: 1_000_000,
        sampler_elements: 500_000,
        hdr_storage: true,
        apple: false,
    };
    let small = Probe {
        sampler_elements: 8,
        ..ready
    };
    let blind = Probe {
        features: WgpuFeatures::empty(),
        ..small
    };
    assert_eq!(check(&ready), Support::Ready);
    assert_eq!(check(&small), Support::TooLimited);
    assert_eq!(check(&blind), Support::NoHardware);
    assert_eq!(missing(blind.features).len(), FEATURES.len());
}

/// An M1 or M2 passes with smaller arrays, whichever sampler limit it reports: 1024 or 996.
#[test]
fn apple_silicon_before_m3_is_ready() {
    for sampler_elements in [1024, 996] {
        let m2 = Probe {
            features: SolariPlugins::required_wgpu_features(),
            binding_elements: 1_000_000,
            sampler_elements,
            hdr_storage: true,
            apple: true,
        };
        assert_eq!(check(&m2), Support::Ready);
        assert!(m2.texture_slots() <= sampler_elements);
        assert!(m2.texture_slots() >= TEXTURE_SLOTS);
    }
}

/// The arrays never ask for more than the device allows, and the element budget follows them.
#[test]
fn arrays_fit_the_device() {
    let wide = Probe {
        features: SolariPlugins::required_wgpu_features(),
        binding_elements: 11_000,
        sampler_elements: 5_000,
        hdr_storage: true,
        apple: false,
    };
    assert_eq!(
        (wide.texture_slots(), wide.elements_needed()),
        (5_000, 11_000)
    );
    assert_eq!(check(&wide), Support::Ready);
    let tight = Probe {
        binding_elements: 10_999,
        ..wide
    };
    assert_eq!(check(&tight), Support::TooLimited);
    for limit in [0, 1, 15, 16, 128, 996, 1024, 4_999, 5_000, 500_000] {
        let probe = Probe {
            sampler_elements: limit,
            ..wide
        };
        assert!(probe.texture_slots() <= limit);
        assert_eq!(check(&probe) == Support::Ready, limit >= TEXTURE_SLOTS);
    }
}
