//! Tests that settings files written by older versions load after an option is removed or moved.

use super::Settings;

/// A settings file that still has the ray tracing option of 0.8 loads: the removed field is
/// skipped and every other value is kept.
#[test]
fn files_with_a_removed_option_load() {
    let old = "[graphics]\npixel_size = 5\nray_tracing = true\nfov = 90\n";
    let parsed: Settings = toml::from_str(old).unwrap();
    assert_eq!((parsed.graphics.pixel_size, parsed.graphics.fov), (5, 90));
    assert_eq!(parsed.controls, Settings::default().controls);
}

/// A file of 0.10 that bound the minimap to M loads with the minimap on N and the map on M,
/// and a file that already names both keys keeps them.
#[test]
fn files_with_the_minimap_on_m_move_it_to_n() {
    use bevy::prelude::KeyCode;
    let old = "[controls.keys]\nminimap = \"KeyM\"\n";
    let keys = toml::from_str::<Settings>(old).unwrap().controls.keys;
    assert_eq!((keys.minimap, keys.map), (KeyCode::KeyN, KeyCode::KeyM));
    let new = "[controls.keys]\nminimap = \"KeyK\"\nmap = \"KeyL\"\n";
    let keys = toml::from_str::<Settings>(new).unwrap().controls.keys;
    assert_eq!((keys.minimap, keys.map), (KeyCode::KeyK, KeyCode::KeyL));
}
