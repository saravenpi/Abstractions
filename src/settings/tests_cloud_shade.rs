//! Tests that the cloud shadows are opt-in: off by default, outside the presets, and off once
//! for a file of an older version that saved them on.

use super::{Graphics, Quality, Settings};

/// The cloud shadows are off by default and outside the presets: no preset turns them on, one
/// turned on by hand stays on through every preset, and the preset still reads as itself.
#[test]
fn cloud_shadows_are_opt_in() {
    assert!(!Graphics::default().lighting.cloud_shade);
    let mut own = Graphics::default();
    for quality in Quality::PRESETS {
        assert!(
            !own.with_quality(quality).lighting.cloud_shade,
            "{quality:?}"
        );
    }
    own.lighting.cloud_shade = true;
    assert_eq!(own.quality(), Quality::High);
    for quality in Quality::PRESETS {
        let set = own.with_quality(quality);
        assert!(
            set.lighting.cloud_shade && set.quality() == quality,
            "{quality:?}"
        );
    }
}

/// A file that saved the old `cloud_shadows` on, the old default, loads with the cloud shadows off
/// once, and saving it writes the new key only.
#[test]
fn an_old_cloud_shadow_flag_is_dropped_once() {
    let old = "[graphics.lighting]\ncloud_shadows = true\nauto_exposure = false\n";
    let parsed: Settings = toml::from_str(old).unwrap();
    assert!(!parsed.graphics.lighting.cloud_shade && !parsed.graphics.lighting.auto_exposure);
    let saved = toml::to_string(&parsed).unwrap();
    assert!(
        saved.contains("cloud_shade = false") && !saved.contains("cloud_shadows"),
        "{saved}"
    );
    let mut chosen = parsed;
    chosen.graphics.lighting.cloud_shade = true;
    let again: Settings = toml::from_str(&toml::to_string(&chosen).unwrap()).unwrap();
    assert!(again.graphics.lighting.cloud_shade);
}
