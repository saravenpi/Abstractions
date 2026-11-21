//! Tests of the HUD at each interface size: the compass and the hotbar stay inside the edge
//! margin of the screen, and the corner strips leave the compass its room.

use bevy::math::Vec2;

use super::needle::WIDTH;
use crate::craft::ui::hotbar::SIDE;
use crate::settings::InterfaceSize;
use crate::ui::{theme, ui_scale};

/// Width in pixels the corner strips of the top row take at most: the coordinates and the minimap.
const CORNERS: f32 = 264.0 + super::minimap::widget::SIDE;

/// At every step in windows from 960 by 540 up, the compass, the hotbar and the two corners fit
/// the logical screen inside the edge margin, and the hotbar clears the bottom edge.
#[test]
fn the_hud_stays_inside_the_edge_margin_at_every_step() {
    let hotbar = 6.0 * SIDE + 5.0 * theme::S2;
    for window in [
        Vec2::new(960.0, 540.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(1920.0, 1080.0),
        Vec2::new(2560.0, 1440.0),
    ] {
        for size in InterfaceSize::ALL {
            let logical = window / ui_scale(window, size);
            let across = logical.x - 2.0 * theme::EDGE;
            assert!(hotbar <= across, "{window} {size:?}");
            assert!(WIDTH + CORNERS <= across, "{window} {size:?}: {logical}");
            let tall = 2.0 * theme::EDGE + SIDE + theme::LINE_SMALL;
            assert!(tall <= logical.y, "{window} {size:?}");
        }
    }
}
