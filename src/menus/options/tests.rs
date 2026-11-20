//! Unit tests for setting steps and key names.

use bevy::prelude::KeyCode;

use super::field::{Bind, Field, set_key, step};
use super::format::{display, key_name};
use super::hint::graphics_line;
use crate::settings::Settings;

/// Numbers clamp at both ends and toggles flip.
#[test]
fn steps_clamp_and_toggle() {
    let mut s = Settings::default();
    for _ in 0..20 {
        step(&mut s, Field::Fov, 1);
        step(&mut s, Field::Master, -1);
        step(&mut s, Field::Sensitivity, 1);
    }
    assert_eq!(s.graphics.fov, 110);
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
        step(&mut s, Field::Fov, -1);
    }
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

/// The two view rows switch between first and third person, each on its own, and the comfort
/// options flip.
#[test]
fn view_rows_switch_and_name_their_mode() {
    let mut s = Settings::default();
    assert_eq!(display(&s, Field::ViewFoot), "First person");
    step(&mut s, Field::ViewFoot, 1);
    assert_eq!(display(&s, Field::ViewFoot), "Third person");
    assert_eq!(display(&s, Field::ViewShip), "First person");
    step(&mut s, Field::ViewShip, -1);
    step(&mut s, Field::ViewFoot, -1);
    assert_eq!(display(&s, Field::ViewShip), "Third person");
    assert_eq!(display(&s, Field::ViewFoot), "First person");
    assert_eq!(display(&s, Field::HeadBob), "On");
    assert_eq!(display(&s, Field::AutoFollow), "On");
    step(&mut s, Field::HeadBob, 1);
    step(&mut s, Field::AutoFollow, 1);
    assert!(!s.view.head_bob && !s.view.auto_follow);
    assert_eq!(display(&s, Field::AutoFollow), "Off");
}

/// The render distance walks the named steps from Low to Extreme and back, shows their names,
/// stays on the ends, and the two highest steps add a note on the GPU to the graphics line.
#[test]
fn render_distance_walks_the_named_steps() {
    let mut s = Settings::default();
    (0..6).for_each(|_| step(&mut s, Field::RenderDistance, -1));
    let mut seen = vec![(
        s.graphics.render_distance,
        display(&s, Field::RenderDistance),
    )];
    for _ in 0..5 {
        step(&mut s, Field::RenderDistance, 1);
        seen.push((
            s.graphics.render_distance,
            display(&s, Field::RenderDistance),
        ));
    }
    let names = ["Low", "Medium", "High", "Ultra", "Extreme", "Extreme"];
    let rings = [3, 4, 8, 16, 24, 24];
    for (k, (got, name)) in seen.iter().enumerate() {
        assert_eq!((*got, name.as_str()), (rings[k], names[k]));
    }
    assert_eq!(graphics_line(8), "");
    assert_eq!(graphics_line(16), "Ultra needs a strong GPU.");
    assert_eq!(graphics_line(24), "Extreme needs a strong GPU.");
}
