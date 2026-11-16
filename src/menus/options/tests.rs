//! Unit tests for setting steps and key names.

use bevy::prelude::KeyCode;

use super::field::{Bind, Field, set_key, step};
use super::format::{display, key_name};
use crate::settings::Settings;

/// Numbers clamp at both ends and toggles flip.
#[test]
fn steps_clamp_and_toggle() {
    let mut s = Settings::default();
    for _ in 0..20 {
        step(&mut s, Field::PixelSize, 1);
        step(&mut s, Field::Master, -1);
        step(&mut s, Field::Sensitivity, 1);
    }
    assert_eq!(s.graphics.pixel_size, 8);
    assert_eq!(s.audio.master, 0.0);
    assert_eq!(s.controls.mouse_sensitivity, 3.0);
    step(&mut s, Field::Shadows, 1);
    assert!(!s.graphics.shadows);
}

/// New graphics fields flip or step in their own ranges.
#[test]
fn steps_new_graphics_fields() {
    let mut s = Settings::default();
    for _ in 0..30 {
        step(&mut s, Field::ColorLevels, 1);
        step(&mut s, Field::Fov, -1);
    }
    assert_eq!(s.graphics.color_levels, 64);
    assert_eq!(s.graphics.fov, 60);
    step(&mut s, Field::Bloom, 1);
    step(&mut s, Field::Clouds, -1);
    step(&mut s, Field::AmbientOcclusion, 1);
    assert!(!s.graphics.bloom && !s.graphics.clouds && !s.graphics.ambient_occlusion);
    assert_eq!(display(&s, Field::Bloom), "Off");
    assert_eq!(display(&s, Field::Fov), "60 deg");
}

/// The frame rate toggle flips and displays as On or Off.
#[test]
fn toggles_show_fps() {
    let mut s = Settings::default();
    assert_eq!(display(&s, Field::ShowFps), "Off");
    step(&mut s, Field::ShowFps, 1);
    assert!(s.graphics.show_fps);
    assert_eq!(display(&s, Field::ShowFps), "On");
}

/// The ray tracing toggle flips, displays as On or Off and starts off.
#[test]
fn toggles_ray_tracing() {
    let mut s = Settings::default();
    assert_eq!(display(&s, Field::RayTracing), "Off");
    step(&mut s, Field::RayTracing, 1);
    assert!(s.graphics.ray_tracing);
    assert_eq!(display(&s, Field::RayTracing), "On");
}

/// Floats move by exactly one tenth and display with one decimal.
#[test]
fn tenths_do_not_drift() {
    let mut s = Settings::default();
    for _ in 0..3 {
        step(&mut s, Field::Music, -1);
    }
    assert_eq!(display(&s, Field::Music), "0.5");
}

/// Keys rebind and print without their Key or Digit prefix.
#[test]
fn rebinds_and_names_keys() {
    let mut s = Settings::default();
    set_key(&mut s, Bind::Jump, KeyCode::KeyF);
    assert_eq!(display(&s, Field::Key(Bind::Jump)), "F");
    assert_eq!(key_name(KeyCode::Digit4), "4");
    assert_eq!(key_name(KeyCode::Space), "Space");
}
