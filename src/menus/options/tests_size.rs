//! Tests of the interface size setting: each step sets the scale it names, the scale lands on
//! whole pixel sizes, the setting survives a save, and the settings page fits the smallest window
//! at the largest step.

use bevy::math::Vec2;

use super::view::{ROWS, TABS};
use crate::settings::{InterfaceSize, Settings};
use crate::ui::{MIN_WIDTH, PAGE_FOOT, PAGE_HEAD, theme, ui_scale};

/// Windows from the smallest supported to a large one.
const WINDOWS: [Vec2; 4] = [
    Vec2::new(960.0, 540.0),
    Vec2::new(1280.0, 720.0),
    Vec2::new(1920.0, 1080.0),
    Vec2::new(2560.0, 1440.0),
];

/// At the 720 pixel reference the steps scale the interface by 0.75, 1 and 1.25, and at 1080 by 1.25,
/// 1.5 and 2, held to what leaves the settings page its width.
#[test]
fn each_step_sets_its_scale() {
    let at = |window: Vec2| InterfaceSize::ALL.map(|size| ui_scale(window, size));
    assert_eq!(at(Vec2::new(1280.0, 720.0)), [0.75, 1.0, 1.25]);
    assert_eq!(at(Vec2::new(1920.0, 1080.0)), [1.25, 1.5, 2.0]);
    assert_eq!(at(Vec2::new(960.0, 540.0)), [0.75, 0.75, 1.0]);
}

/// Every scale is a multiple of a quarter, so the sizes of the type scale and every position land
/// on whole pixels, and the smallest step keeps 16 pixel text at 12 pixels or more.
#[test]
fn every_scale_keeps_text_on_whole_pixels() {
    for window in WINDOWS {
        for size in InterfaceSize::ALL {
            let scale = ui_scale(window, size);
            assert_eq!(scale * 4.0, (scale * 4.0).round(), "{window} {size:?}");
            for text in [16.0, 20.0, 32.0, 72.0] {
                assert_eq!(text * scale, (text * scale).round(), "{text} at {scale}");
            }
            assert!(16.0 * scale >= 12.0, "{window} {size:?}");
        }
    }
}

/// The step is saved with the settings, and a file from before it loads with the usual size.
#[test]
fn the_step_survives_a_save_and_old_files_load_normal() {
    let mut settings = Settings::default();
    settings.gameplay.interface = InterfaceSize::Large;
    let saved = toml::to_string(&settings).unwrap();
    let loaded: Settings = toml::from_str(&saved).unwrap();
    assert_eq!(loaded.gameplay.interface, InterfaceSize::Large);
    let old: Settings = toml::from_str("[gameplay]\nhitstop = 0.5\n").unwrap();
    assert_eq!(old.gameplay.interface, InterfaceSize::Normal);
}

/// The stepper walks the three steps and stops on the first and the last.
#[test]
fn the_stepper_walks_the_steps() {
    let mut settings = Settings::default();
    let mut press = |dir| {
        super::field::step(&mut settings, super::field::Field::Interface, dir);
        settings.gameplay.interface
    };
    assert_eq!(press(1), InterfaceSize::Large);
    assert_eq!(press(1), InterfaceSize::Large);
    assert_eq!(press(-1), InterfaceSize::Normal);
    assert_eq!(press(-1), InterfaceSize::Small);
    assert_eq!(press(-1), InterfaceSize::Small);
}

/// Fewest rows of a page that must show at once, whatever the size of the interface.
const MIN_ROWS: f32 = 6.0;

/// At every step and in every window, from 960 by 540 up, the settings page has the width of its
/// tabs, a gutter and its page inside the screen edges, and its scrolling column keeps the height
/// of its fewest rows between the heading zone, the line under the rows and the footer zone.
#[test]
fn the_settings_page_fits_at_every_step() {
    let needed = theme::span(TABS) + theme::GUTTER + theme::span(ROWS) + 2.0 * theme::EDGE;
    assert_eq!(theme::span(TABS + ROWS), needed - 2.0 * theme::EDGE);
    assert!(needed <= MIN_WIDTH);
    let line = theme::S3 + theme::LINE_SMALL;
    let chrome = PAGE_HEAD + line + PAGE_FOOT;
    for window in WINDOWS {
        for size in InterfaceSize::ALL {
            let logical = window / ui_scale(window, size);
            assert!(logical.x >= needed, "{window} {size:?}: {logical}");
            let rows = ((logical.y - chrome) / theme::ROW_HEIGHT).floor();
            assert!(rows >= MIN_ROWS, "{window} {size:?}: {rows} rows");
        }
    }
}
