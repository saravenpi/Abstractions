//! Tests that settings files written by older versions load after an option is removed or moved.

use super::Settings;

/// A settings file that still has the ray tracing option of 0.8 loads: the removed field is
/// skipped and every other value is kept.
#[test]
fn files_with_a_removed_option_load() {
    let old = "[graphics]\nrender_distance = 8\nray_tracing = true\nfov = 90\n";
    let parsed: Settings = toml::from_str(old).unwrap();
    assert_eq!(
        (parsed.graphics.render_distance, parsed.graphics.fov),
        (8, 90)
    );
    assert_eq!(parsed.controls, Settings::default().controls);
}

/// A file of 0.10 or older that still sets the pixel size and the colour levels loads: both
/// fields are skipped, every other value is kept, and saving it again drops them.
#[test]
fn files_with_the_pixel_options_load() {
    let old = "[graphics]\npixel_size = 5\ncolor_levels = 40\nfov = 90\nbloom = false\n";
    let parsed: Settings = toml::from_str(old).unwrap();
    assert_eq!((parsed.graphics.fov, parsed.graphics.bloom), (90, false));
    let saved = toml::to_string(&parsed).unwrap();
    assert!(
        !saved.contains("pixel_size") && !saved.contains("color_levels"),
        "{saved}"
    );
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

/// A file of 0.11 written while the first-time hints existed still loads: their bits are skipped,
/// every other value is kept, and saving it again drops them.
#[test]
fn files_with_the_hint_bits_load() {
    let old = "hints = 7\nplayer_name = \"Ada\"\n\n[graphics]\nfov = 90\n";
    let parsed: Settings = toml::from_str(old).unwrap();
    assert_eq!(
        (parsed.player_name.as_str(), parsed.graphics.fov),
        ("Ada", 90)
    );
    let saved = toml::to_string(&parsed).unwrap();
    assert!(!saved.contains("hints"), "{saved}");
}
