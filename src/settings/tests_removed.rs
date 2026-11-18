//! Tests that settings files written by older versions load after an option is removed.

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
