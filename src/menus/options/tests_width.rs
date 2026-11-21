//! Width tests, measured with the interface font: every value a setting can show fits the value
//! column, every key name fits it with its conflict mark, and every line under the rows wraps in
//! the lines the page keeps for it.

use bevy::input::keyboard::NativeKeyCode;
use bevy::prelude::KeyCode;

use super::binds::Bind;
use super::describe::{COSTS, describe, describe_general};
use super::field::{Field, set_key, step};
use super::format::display;
use super::hint::{conflict_line, graphics_line};
use super::names::{ELLIPSIS, KEY_CHARS, key_label, key_name};
use super::tests_size::HINT_LINES;
use super::view::ROWS;
use crate::menus::tests::stage::lines;
use crate::settings::Settings;
use crate::ui::{text_width, theme};

/// Every setting that is not a key.
const FIELDS: [Field; 35] = [
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
    Field::Range,
    Field::Mono,
    Field::CompanionVoice,
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

/// Keys with long names: the ones the settings name in words, and the widest a keyboard has,
/// up to a key the system does not know.
const LONG_KEYS: [KeyCode; 18] = [
    KeyCode::ShiftRight,
    KeyCode::ControlRight,
    KeyCode::SuperRight,
    KeyCode::PageDown,
    KeyCode::PrintScreen,
    KeyCode::ScrollLock,
    KeyCode::NumpadEnter,
    KeyCode::NumpadParenRight,
    KeyCode::NumpadBackspace,
    KeyCode::NumpadMemorySubtract,
    KeyCode::NumpadClearEntry,
    KeyCode::BracketRight,
    KeyCode::AudioVolumeDown,
    KeyCode::BrowserFavorites,
    KeyCode::MediaTrackPrevious,
    KeyCode::IntlBackslash,
    KeyCode::LaunchApp1,
    KeyCode::Unidentified(NativeKeyCode::MacOS(65535)),
];

/// The names planned for the render distance steps, which the column must also hold.
const PLANNED: [&str; 5] = ["Low", "Medium", "High", "Ultra", "Extreme"];

/// Returns the width of `words` in body text.
fn body(words: &str) -> f32 {
    text_width(words, theme::FONT_BODY, 0.0)
}

/// Returns every text `field` shows over its whole range.
fn shown(field: Field) -> Vec<String> {
    let mut settings = Settings::default();
    let mut seen = Vec::new();
    for dir in [-1, 1] {
        for _ in 0..300 {
            seen.push(display(&settings, field));
            step(&mut settings, field, dir);
        }
    }
    seen
}

/// The rig reads the font as the game draws it: a letter of body text advances 12 pixels, a
/// fraction twice that, and spacing adds after each glyph.
#[test]
fn the_rig_measures_the_interface_font() {
    assert_eq!(body("Left Shift"), 120.0);
    assert_eq!(body("\u{bd}"), 24.0);
    let spaced = text_width("AB", theme::FONT_SMALL, theme::CAPS_SPACING);
    assert!(
        (spaced - 2.0 * (9.6 + theme::CAPS_SPACING)).abs() < 1e-3,
        "{spaced}"
    );
}

/// Every value of every setting over its whole range, and every planned step name, fits the
/// value column.
#[test]
fn every_value_fits_the_value_column() {
    for field in FIELDS {
        for text in shown(field) {
            assert!(body(&text) <= theme::VALUE_WIDTH, "{field:?}: {text}");
        }
    }
    assert_eq!(
        shown(Field::Fov)
            .iter()
            .map(|text| body(text))
            .fold(0.0, f32::max),
        body("110 deg")
    );
    for name in PLANNED {
        assert!(body(name) <= theme::VALUE_WIDTH, "{name}");
    }
}

/// Every key name fits the value column with the mark of a conflict after it. A name too long
/// for it keeps its start and ends with an ellipsis, and the full name stays for the tooltip.
#[test]
fn every_key_name_fits_with_its_conflict_mark() {
    for key in LONG_KEYS {
        let (name, full) = (key_name(key), key_label(key));
        assert!(body(&format!("{name} !")) <= theme::VALUE_WIDTH, "{name}");
        if full.chars().count() <= KEY_CHARS {
            assert_eq!(name, full);
            continue;
        }
        let kept = name.strip_suffix(ELLIPSIS).expect("an ellipsis");
        assert!(full.starts_with(kept), "{name} cuts {full}");
    }
    assert_eq!(key_name(KeyCode::ControlLeft), "Left Ctrl");
    assert_eq!(key_name(KeyCode::PageDown), "Page Down");
    assert_eq!(
        key_label(KeyCode::NumpadMemorySubtract),
        "Num MemorySubtract"
    );
}

/// Every line under the rows wraps in the lines the page keeps for it: each description, the
/// render distance note after its description, what costs most, and a conflict of two long
/// action names on a key with a long name.
#[test]
fn every_line_under_the_rows_wraps_in_its_lines() {
    let mut said: Vec<String> = FIELDS
        .into_iter()
        .flat_map(|field| [describe(field), describe_general(field)])
        .map(str::to_string)
        .collect();
    said.push(COSTS.to_string());
    said.push(format!(
        "{} {}",
        describe(Field::RenderDistance),
        graphics_line(24)
    ));
    let mut settings = Settings::default();
    set_key(&mut settings, Bind::Sprint, KeyCode::NumpadMemorySubtract);
    set_key(&mut settings, Bind::Board, KeyCode::NumpadMemorySubtract);
    said.push(conflict_line(&settings.controls.keys));
    let room = theme::span(ROWS) - theme::INSET;
    let space = text_width(" ", theme::FONT_SMALL, 0.0);
    for line in said {
        let words: Vec<f32> = line
            .split_whitespace()
            .map(|word| text_width(word, theme::FONT_SMALL, 0.0))
            .collect();
        let taken = lines(&words, space, room);
        assert!(taken <= HINT_LINES, "{taken} lines: {line}");
    }
}
