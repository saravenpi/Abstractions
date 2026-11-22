//! Tests of the lighting rows of the graphics tab: they step, show their value and describe
//! themselves, and a preset step moves the options that belong to it and not the cloud shadows.

use super::describe::{COSTS, describe};
use super::field::{Field, step};
use super::hint::graphics_line;
use super::rig::display;
use crate::settings::Settings;

/// The preset walks from High to Ultra and Low, and one option changed by hand reads Custom.
#[test]
fn the_preset_row_walks_and_reads_custom() {
    let mut s = Settings::default();
    assert_eq!(display(&s, Field::LightingQuality), "High");
    step(&mut s, Field::LightingQuality, 1);
    assert_eq!(display(&s, Field::LightingQuality), "Ultra");
    assert!(s.graphics.lighting.local_shadows);
    (0..3).for_each(|_| step(&mut s, Field::LightingQuality, -1));
    assert_eq!(display(&s, Field::LightingQuality), "Low");
    assert!(!s.graphics.shadows && !s.graphics.bloom);
    step(&mut s, Field::AutoExposure, 1);
    assert_eq!(display(&s, Field::LightingQuality), "Custom");
}

/// The cloud shadows start off and step on without moving the preset: they are opt-in.
#[test]
fn the_cloud_shadow_row_is_outside_the_preset() {
    let mut s = Settings::default();
    assert_eq!(display(&s, Field::CloudShadows), "Off");
    step(&mut s, Field::CloudShadows, 1);
    assert_eq!(display(&s, Field::CloudShadows), "On");
    assert_eq!(display(&s, Field::LightingQuality), "High");
}

/// Every lighting row shows its two or three values and steps between them, and each one has a
/// description of a plain sentence.
#[test]
fn the_lighting_rows_show_their_values() {
    let mut s = Settings::default();
    assert_eq!(display(&s, Field::ShadowDistance), "110 m");
    step(&mut s, Field::ShadowDistance, 1);
    assert_eq!(display(&s, Field::ShadowDistance), "220 m");
    step(&mut s, Field::ShadowDistance, 1);
    assert_eq!(display(&s, Field::ShadowDistance), "220 m");
    step(&mut s, Field::ShadowDetail, 1);
    assert_eq!(display(&s, Field::ShadowDetail), "Low");
    assert_eq!(display(&s, Field::WaterGlint), "Sky and sun");
    step(&mut s, Field::WaterGlint, 1);
    assert_eq!(display(&s, Field::WaterGlint), "Sky only");
    for field in [Field::AutoExposure, Field::LocalShadows, Field::Shadows] {
        assert!(!describe(field).is_empty(), "{field:?}");
        assert!(describe(field).len() <= 80, "{field:?}");
    }
    assert!(COSTS.contains("shadows") && graphics_line(4).is_empty());
}

/// The water reflections row reads Actors on the default High preset, cycles to Off, which reads
/// as a custom preset, and back; Low turns it off and Ultra on.
#[test]
fn the_water_reflections_row_cycles_and_follows_the_preset() {
    let mut s = Settings::default();
    assert_eq!(display(&s, Field::WaterReflections), "Actors");
    step(&mut s, Field::WaterReflections, 1);
    assert_eq!(display(&s, Field::WaterReflections), "Off");
    assert_eq!(display(&s, Field::LightingQuality), "Custom");
    step(&mut s, Field::WaterReflections, -1);
    assert_eq!(display(&s, Field::WaterReflections), "Actors");
    assert_eq!(display(&s, Field::LightingQuality), "High");
    (0..3).for_each(|_| step(&mut s, Field::LightingQuality, -1));
    assert_eq!(display(&s, Field::WaterReflections), "Off");
    (0..3).for_each(|_| step(&mut s, Field::LightingQuality, 1));
    assert_eq!(display(&s, Field::WaterReflections), "Actors");
    assert!(!describe(Field::WaterReflections).is_empty());
    assert!(describe(Field::WaterReflections).len() <= 80);
}
