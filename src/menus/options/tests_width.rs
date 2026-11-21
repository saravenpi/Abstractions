//! Width tests: the widest value of every setting fits the fixed value column, so none wraps.

use bevy::prelude::KeyCode;

use super::binds::Bind;
use super::field::{Field, step};
use super::format::{KEY_CHARS, display, key_name};
use super::reference::ACTION_CHARS;
use super::view::ROWS;
use crate::settings::Settings;
use crate::ui::theme;

/// Every setting that is not a key.
const FIELDS: [Field; 32] = [
    Field::RenderDistance,
    Field::LightingQuality,
    Field::Shadows,
    Field::ShadowDetail,
    Field::ShadowDistance,
    Field::CloudShadows,
    Field::AutoExposure,
    Field::WhiteBalance,
    Field::LocalShadows,
    Field::WaterGlint,
    Field::Vsync,
    Field::Fullscreen,
    Field::AmbientOcclusion,
    Field::Bloom,
    Field::Clouds,
    Field::ShowFps,
    Field::Fov,
    Field::Sensitivity,
    Field::InvertY,
    Field::Master,
    Field::Music,
    Field::Effects,
    Field::ViewFoot,
    Field::ViewShip,
    Field::HeadBob,
    Field::AutoFollow,
    Field::CheckUpdates,
    Field::Hitstop,
    Field::WorldMarkers,
    Field::Blood,
    Field::Interface,
    Field::ReduceMotion,
];

/// Keys with long names, among them the widest ones a keyboard has.
const LONG_KEYS: [KeyCode; 12] = [
    KeyCode::ControlLeft,
    KeyCode::ControlRight,
    KeyCode::ShiftRight,
    KeyCode::NumpadEnter,
    KeyCode::NumpadSubtract,
    KeyCode::NumpadParenRight,
    KeyCode::NumpadMemorySubtract,
    KeyCode::BracketRight,
    KeyCode::AudioVolumeDown,
    KeyCode::BrowserFavorites,
    KeyCode::IntlBackslash,
    KeyCode::ScrollLock,
];

/// The longest name a settings row shows.
const LONGEST: &str = "Markers over the world";

/// The names planned for the render distance steps, which the column must also hold.
const PLANNED: [&str; 5] = ["Low", "Medium", "High", "Ultra", "Extreme"];

/// Returns the most characters `field` shows over its whole range.
fn widest(field: Field) -> usize {
    let mut settings = Settings::default();
    let mut most = 0;
    for dir in [-1, 1] {
        for _ in 0..300 {
            most = most.max(display(&settings, field).chars().count());
            step(&mut settings, field, dir);
        }
    }
    most
}

/// The widest value of every setting, every long key name and every planned step name fits the
/// character budget of the value column.
#[test]
fn every_value_fits_the_value_column() {
    for field in FIELDS {
        assert!(widest(field) <= theme::VALUE_CHARS, "{field:?}");
    }
    assert_eq!(widest(Field::Fov), "110 deg".len());
    for key in LONG_KEYS {
        let name = key_name(key);
        assert!(name.chars().count() <= KEY_CHARS, "{name}");
    }
    assert!(PLANNED.iter().all(|name| name.len() <= theme::VALUE_CHARS));
    assert_eq!(key_name(KeyCode::ControlLeft), "Left Ctrl");
}

/// The value column is wide enough for its character budget in body text, and a settings row
/// with the longest name still fits the page: its icon, its name and its value zone, inside the
/// padding of the row.
#[test]
fn the_value_column_holds_its_budget_and_the_longest_row_fits() {
    let glyph = theme::GLYPH_ASPECT * theme::FONT_BODY;
    assert!(theme::VALUE_CHARS as f32 * glyph <= theme::VALUE_WIDTH);
    assert_eq!(theme::VALUE_CLUSTER, theme::span(3) + theme::S3);
    let name = theme::ICON + theme::ICON_GAP + LONGEST.len() as f32 * glyph;
    let row = name + theme::S3 + theme::VALUE_CLUSTER + 2.0 * theme::INSET;
    assert!(row <= theme::span(ROWS), "{row}");
}

/// A line of the reference fits the page too: an action, then a key of the widest name.
#[test]
fn a_reference_line_fits_the_page() {
    let glyph = theme::GLYPH_ASPECT * theme::FONT_SMALL;
    assert!(KEY_CHARS as f32 * glyph <= theme::VALUE_CLUSTER);
    let action = theme::ICON + theme::ICON_GAP + ACTION_CHARS as f32 * glyph;
    let line = action + theme::S3 + theme::VALUE_CLUSTER + 2.0 * theme::INSET;
    assert!(line <= theme::span(ROWS), "{line}");
}

/// Every action of the controls reference stays on one line.
#[test]
fn reference_actions_fit_one_line() {
    let binds = Bind::FOOT.into_iter().chain(Bind::SHIP);
    for bind in binds {
        assert!(bind.label().len() <= ACTION_CHARS, "{}", bind.label());
    }
}
