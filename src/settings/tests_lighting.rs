//! Tests of the lighting presets: each one maps to its documented options, the lights stay, and
//! a file written before the options existed still loads.

use super::{Graphics, Lighting, Quality, Settings, ShadowDetail, ShadowReach};

/// Returns the graphics options of preset `quality` on top of the defaults.
fn preset(quality: Quality) -> Graphics {
    Graphics::default().with_quality(quality)
}

/// Low drops the shadows, the occlusion, the bloom, the cloud shadows, the exposure and the glint.
#[test]
fn low_drops_every_costly_effect() {
    let low = preset(Quality::Low);
    assert!(!low.shadows && !low.ambient_occlusion && !low.bloom);
    assert!(!low.lighting.cloud_shade && !low.lighting.auto_exposure);
    assert!(!low.lighting.local_shadows && !low.lighting.water_glint);
    assert_eq!(low.lighting.shadow_detail, ShadowDetail::Low);
    assert_eq!(low.lighting.shadow_reach, ShadowReach::Near);
}

/// Medium has shadows, bloom and exposure, and no occlusion, no cloud shadows, no lamp shadows.
#[test]
fn medium_keeps_the_cheap_effects() {
    let medium = preset(Quality::Medium);
    assert!(medium.shadows && medium.bloom && medium.lighting.auto_exposure);
    assert!(!medium.ambient_occlusion && !medium.lighting.cloud_shade);
    assert!(!medium.lighting.local_shadows);
    assert_eq!(medium.lighting.shadow_detail, ShadowDetail::Low);
}

/// High is the default, and Ultra adds the far shadows and the lamp shadows.
#[test]
fn high_is_the_default_and_ultra_adds_reach() {
    assert_eq!(Graphics::default().quality(), Quality::High);
    assert_eq!(Lighting::default(), preset(Quality::High).lighting);
    let ultra = preset(Quality::Ultra);
    assert_eq!(ultra.lighting.shadow_reach, ShadowReach::Far);
    assert!(ultra.lighting.local_shadows && ultra.ambient_occlusion);
    assert_eq!(ultra.lighting.shadow_reach.cascades(), 4);
    assert_eq!(preset(Quality::High).lighting.shadow_reach.cascades(), 3);
    assert_eq!(ShadowDetail::Low.map_size(), 1024);
    assert_eq!(ShadowDetail::High.map_size(), 2048);
    let reach = [ShadowReach::Near, ShadowReach::Normal, ShadowReach::Far];
    assert_eq!(reach.map(ShadowReach::metres), [55.0, 110.0, 220.0]);
}

/// A preset keeps what is not lighting, and each one reads back as itself.
#[test]
fn presets_keep_the_rest_and_read_back() {
    let mut own = Graphics::default();
    (own.render_distance, own.fov, own.vsync, own.clouds) = (16, 100, false, false);
    for quality in Quality::PRESETS {
        let set = own.with_quality(quality);
        assert_eq!(set.quality(), quality);
        assert_eq!(
            (set.render_distance, set.fov, set.vsync, set.clouds),
            (16, 100, false, false)
        );
    }
}

/// One option changed by hand makes the preset read as custom, and a step leaves it.
#[test]
fn a_single_change_is_custom() {
    let mut own = Graphics::default();
    own.lighting.auto_exposure = false;
    assert_eq!(own.quality(), Quality::Custom);
    assert_eq!(own.with_quality(Quality::Custom), own);
    own.step_quality(1);
    assert_eq!(own.quality(), Quality::Ultra);
    let mut low = Graphics::default();
    low.lighting.auto_exposure = false;
    low.step_quality(-1);
    assert_eq!(low.quality(), Quality::Low);
}

/// Steps stop on the first and the last preset.
#[test]
fn steps_stop_at_the_ends() {
    let mut own = Graphics::default();
    (0..6).for_each(|_| own.step_quality(-1));
    assert_eq!(own.quality(), Quality::Low);
    (0..6).for_each(|_| own.step_quality(1));
    assert_eq!(own.quality(), Quality::Ultra);
}

/// A settings file without the lighting table loads with the High options.
#[test]
fn an_old_file_gets_the_default_lighting() {
    let old: Settings = toml::from_str("[graphics]\nfov = 90\nshadows = false\n").unwrap();
    assert_eq!(old.graphics.lighting, Lighting::default());
    assert!(!old.graphics.shadows);
    let text = toml::to_string(&Settings::default()).unwrap();
    assert!(text.contains("[graphics.lighting]"));
}
