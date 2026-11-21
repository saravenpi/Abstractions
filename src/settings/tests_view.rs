//! Tests of the view preferences kept in the settings.

use super::Settings;

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
