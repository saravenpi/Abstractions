//! Tests that a slot of the hotbar of the game holds its number, the 24 pixel icon of its tool
//! and its wear bar without overlap at every interface size, and that the icon lands on whole
//! pixels there.

use bevy::math::Vec2;

use super::glyph::{HUD, snap};
use super::hotbar::{BAR, GAP, NUMBER, SIDE};
use crate::settings::InterfaceSize;
use crate::ui::{METER, theme, ui_scale};

/// The icon and the wear bar stack inside the slot with room at the top and the foot; the number
/// in its corner stays left of the icon; the bar fits across.
#[test]
fn a_slot_holds_its_number_icon_and_wear() {
    let stack = HUD + GAP + METER;
    let room = (SIDE - stack) / 2.0;
    assert!(room >= theme::S2, "{room}");
    let number = theme::FONT_SMALL * theme::GLYPH_ASPECT;
    assert!(
        NUMBER + number <= (SIDE - HUD) / 2.0,
        "the number runs into the icon"
    );
    assert!(BAR <= SIDE - 2.0 * theme::S1);
    assert!(HUD >= theme::ICON);
}

/// At every window and interface size the icon of a slot covers whole pixels at its own size,
/// and the six slots fit across the screen inside the edge margin.
#[test]
fn the_hotbar_icon_is_pixel_aligned_and_fits() {
    let windows = [
        Vec2::new(960.0, 540.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(1920.0, 1080.0),
        Vec2::new(2560.0, 1440.0),
    ];
    for window in windows {
        for size in InterfaceSize::ALL {
            let scale = ui_scale(window, size);
            let (pixels, logical) = snap(HUD, scale);
            assert_eq!(logical, HUD, "{window} {size:?}");
            assert_eq!(pixels as f32, HUD * scale, "{window} {size:?}");
            let across = window.x / scale - 2.0 * theme::EDGE;
            assert!(6.0 * SIDE + 5.0 * theme::S2 <= across, "{window} {size:?}");
        }
    }
}
