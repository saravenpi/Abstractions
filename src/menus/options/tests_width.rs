//! Width tests: the widest value of every setting fits the fixed value column, so none wraps.

use bevy::prelude::KeyCode;

use super::binds::Bind;
use super::field::{Field, step};
use super::format::{KEY_CHARS, display, key_name};
use super::reference::ACTION_CHARS;
use crate::settings::Settings;
use crate::ui::theme;

/// Every setting that is not a key.
const FIELDS: [Field; 22] = [
    Field::PixelSize,
    Field::RenderDistance,
    Field::Shadows,
    Field::Vsync,
    Field::Fullscreen,
    Field::AmbientOcclusion,
    Field::Bloom,
    Field::Clouds,
    Field::ShowFps,
    Field::ColorLevels,
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
/// with the longest name still fits the page at the smallest supported window.
#[test]
fn the_value_column_holds_its_budget_at_the_smallest_window() {
    let glyph = theme::GLYPH_ASPECT * theme::FONT_BODY;
    assert!(theme::VALUE_CHARS as f32 * glyph <= theme::VALUE_WIDTH);
    let stepper = theme::VALUE_WIDTH + 2.0 * (theme::GAP_L + theme::GAP_XS);
    let name = "Camera auto-follow".len() as f32 * glyph;
    let page = theme::MENU_WIDTH * 2.0;
    assert!(name + theme::GAP_M + stepper + 2.0 * theme::GAP_S <= page);
    let sidebar = theme::MENU_WIDTH * 0.6 + theme::GAP_XL;
    let panel = 2.0 * (theme::GAP_M + theme::LINE);
    assert!(
        sidebar + page + panel <= 960.0 * 0.9,
        "the menu fits a 960 pixel window"
    );
}

/// Every action of the controls reference stays on one line.
#[test]
fn reference_actions_fit_one_line() {
    let binds = Bind::FOOT.into_iter().chain(Bind::SHIP);
    for bind in binds {
        assert!(bind.label().len() <= ACTION_CHARS, "{}", bind.label());
    }
}
