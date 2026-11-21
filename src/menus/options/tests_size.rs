//! Tests of the interface size setting: each step sets the scale it names, the scale lands on
//! whole pixel sizes, the setting survives a save, and the settings page fits the smallest window
//! at the largest step.

use bevy::math::Vec2;

use super::view::{CHROME, MIN_ROWS, rows_for};
use crate::settings::{InterfaceSize, Settings};
use crate::ui::{MIN_WIDTH, theme, ui_scale};

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

/// At every step and in every window, from 960 by 540 up, the settings page has the width of its
/// sidebar, page and panel across, and shows at least its fewest rows inside the height.
#[test]
fn the_settings_page_fits_at_every_step() {
    let sidebar = theme::MENU_WIDTH * 0.6 + theme::GAP_XL;
    let panel = 2.0 * (theme::GAP_M + theme::LINE);
    let needed = (sidebar + theme::MENU_WIDTH * 2.0 + panel) / 0.9;
    assert!(needed <= MIN_WIDTH);
    for window in WINDOWS {
        for size in InterfaceSize::ALL {
            let logical = window / ui_scale(window, size);
            assert!(logical.x >= needed, "{window} {size:?}: {logical}");
            let rows = rows_for(logical.y);
            assert!(rows >= MIN_ROWS);
            let room = CHROME + rows * theme::ROW_HEIGHT;
            assert!(room <= logical.y || rows == MIN_ROWS, "{window} {size:?}");
            assert!(
                CHROME + MIN_ROWS * theme::ROW_HEIGHT <= logical.y,
                "{window} {size:?}"
            );
        }
    }
}
