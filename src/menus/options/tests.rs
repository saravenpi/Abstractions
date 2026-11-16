//! Unit tests for setting steps and key names.

use bevy::prelude::KeyCode;

use super::field::{Bind, Field, display, key_name, set_key, step};
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
