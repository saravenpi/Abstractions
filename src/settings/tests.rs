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

/// A partial file keeps its values and fills the rest with defaults.
#[test]
fn partial_file_uses_defaults() {
    let parsed: Settings = toml::from_str("[graphics]\npixel_size = 6\n").unwrap();
    assert_eq!(parsed.graphics.pixel_size, 6);
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

/// The default bindings of the computer match the constants the computer documents.
#[test]
fn computer_keys_default_to_tab_and_g() {
    let keys = Settings::default().controls.keys;
    assert_eq!(keys.computer, crate::computer::OPEN_KEY);
    assert_eq!(keys.engage, crate::computer::ENGAGE_KEY);
}
