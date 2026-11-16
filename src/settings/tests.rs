//! Settings serialization and fallback tests.

use super::{Settings, load_from};

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
