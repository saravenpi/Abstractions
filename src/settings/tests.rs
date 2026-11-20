//! Settings serialization and fallback tests.

use bevy::prelude::KeyCode;

use super::{Settings, ShipKeys, load_from};

/// Returns a fresh scratch file path for one test.
fn scratch(name: &str) -> std::path::PathBuf {
    std::env::temp_dir()
        .join(format!("abstractions-test-{}-{name}", std::process::id()))
        .join("settings.toml")
}

/// Defaults survive a TOML round trip.
#[test]
fn default_round_trips() {
    let text = toml::to_string(&Settings::default()).unwrap();
    assert_eq!(
        toml::from_str::<Settings>(&text).unwrap(),
        Settings::default()
    );
}

/// A new profile starts in third person on foot and first person in the ship, and a toggled view
/// survives a round trip of the settings.
#[test]
fn the_view_starts_behind_the_body_and_a_toggle_is_kept() {
    use crate::view::{Kind, ViewMode};
    let mut settings = Settings::default();
    assert_eq!(settings.view.on_foot, ViewMode::Third);
    assert_eq!(settings.view.ship, ViewMode::First);
    settings.view.toggle(Kind::OnFoot);
    let text = toml::to_string(&settings).unwrap();
    let back: Settings = toml::from_str(&text).unwrap();
    assert_eq!(back.view.on_foot, ViewMode::First);
    assert_eq!(back.view.ship, ViewMode::First);
}

/// A partial file keeps its values and fills the rest with defaults.
#[test]
fn partial_file_uses_defaults() {
    let parsed: Settings = toml::from_str("[graphics]\nfov = 95\n").unwrap();
    assert_eq!(parsed.graphics.fov, 95);
    assert_eq!(
        parsed.graphics.render_distance,
        Settings::default().graphics.render_distance
    );
    assert_eq!(parsed.audio, Settings::default().audio);
}

/// A broken file on disk loads as defaults instead of failing.
#[test]
fn broken_file_falls_back_to_defaults() {
    let path = scratch("broken");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "graphics = [not toml").unwrap();
    let loaded = load_from(path.clone());
    assert_eq!(*loaded.get(), Settings::default());
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

/// A settings file exactly as v0.5 wrote it: no ship keys, no computer keys.
const V05: &str = r#"[graphics]
pixel_size = 4
render_distance = 6
shadows = true
vsync = false
fullscreen = false
ambient_occlusion = true
bloom = true
clouds = true
color_levels = 24
fov = 80
show_fps = true
ray_tracing = false

[controls]
mouse_sensitivity = 1.5
invert_y = true

[controls.keys]
forward = "KeyZ"
back = "KeyS"
left = "KeyQ"
right = "KeyD"
jump = "Space"
sprint = "ShiftLeft"

[audio]
master = 0.5
music = 0.25
effects = 0.75

[world]
last_seed = 42
"#;

/// A v0.5 file keeps every value it has and gets the default ship, computer and engage keys.
#[test]
fn a_v05_file_loads_with_the_new_keys_defaulted() {
    let parsed: Settings = toml::from_str(V05).unwrap();
    let keys = parsed.controls.keys;
    assert_eq!((keys.forward, keys.left), (KeyCode::KeyZ, KeyCode::KeyQ));
    assert_eq!((keys.computer, keys.engage), (KeyCode::Tab, KeyCode::KeyG));
    assert_eq!(keys.ship, ShipKeys::default());
    assert_eq!(parsed.graphics.fov, 80);
    assert_eq!(parsed.audio.music, 0.25);
    assert_eq!(parsed.world.last_seed, Some(42));
    let path = scratch("v05");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, V05).unwrap();
    assert_eq!(*load_from(path.clone()).get(), parsed);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

/// The board computer opens on Tab and engages on G by default.
#[test]
fn computer_keys_default_to_tab_and_g() {
    let keys = Settings::default().controls.keys;
    assert_eq!(keys.computer, KeyCode::Tab);
    assert_eq!(keys.engage, KeyCode::KeyG);
}

/// A file written before the camera and flight settings existed, and a v0.5 file, get the
/// default view preferences and the default wingsuit and look keys, and keep everything else.
#[test]
fn files_without_the_view_settings_load_with_defaults() {
    let mut text = toml::to_string(&Settings {
        audio: crate::settings::Audio {
            music: 0.3,
            ..Default::default()
        },
        ..Settings::default()
    })
    .unwrap();
    let start = text.find("[view]").unwrap();
    let end = text[start + 1..]
        .find("\n[")
        .map_or(text.len(), |at| start + 1 + at);
    text.replace_range(start..end, "");
    let text: String = text
        .lines()
        .filter(|line| !line.starts_with("wings =") && !line.starts_with("orbit ="))
        .map(|line| format!("{line}\n"))
        .collect();
    for old in [text.as_str(), V05] {
        let parsed: Settings = toml::from_str(old).unwrap();
        assert_eq!(parsed.view, crate::view::ViewPrefs::default());
        let keys = parsed.controls.keys;
        assert_eq!((keys.wings, keys.view), (KeyCode::KeyR, KeyCode::KeyV));
    }
    assert_eq!(toml::from_str::<Settings>(&text).unwrap().audio.music, 0.3);
}

/// A render distance from a file written before the named steps moves to the nearest step, the
/// lower one when it lies halfway; a step itself stays.
#[test]
fn old_render_distances_move_to_the_nearest_step() {
    let read = |rings: u32| {
        let text = format!("[graphics]\nrender_distance = {rings}\n");
        toml::from_str::<Settings>(&text)
            .unwrap()
            .graphics
            .render_distance
    };
    let pairs = [
        (1, 3),
        (2, 3),
        (5, 4),
        (6, 4),
        (7, 8),
        (12, 8),
        (13, 16),
        (20, 16),
        (30, 24),
    ];
    for (old, step) in pairs {
        assert_eq!(read(old), step, "{old}");
    }
    for step in [3, 4, 8, 16, 24] {
        assert_eq!(read(step), step);
    }
}
