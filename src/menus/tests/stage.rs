//! Test stage: the windows the fit tests lay pages out in, and the layout each gives at each
//! interface size.

use bevy::math::Vec2;

use crate::settings::InterfaceSize;
use crate::ui::{MIN_WIDTH, ui_scale};

/// Logical windows from the smallest the game allows to large ones: the usual laptop and desktop
/// windows, an ultrawide one, and a window that is tall for its width.
pub const WINDOWS: [Vec2; 12] = [
    Vec2::new(960.0, 540.0),
    Vec2::new(1024.0, 768.0),
    Vec2::new(1280.0, 720.0),
    Vec2::new(1280.0, 800.0),
    Vec2::new(1366.0, 768.0),
    Vec2::new(1440.0, 900.0),
    Vec2::new(1600.0, 900.0),
    Vec2::new(1920.0, 1080.0),
    Vec2::new(1920.0, 1200.0),
    Vec2::new(2560.0, 1080.0),
    Vec2::new(2560.0, 1440.0),
    Vec2::new(960.0, 1200.0),
];

/// Physical windows of laptops with a Retina screen, drawn at two physical pixels to a logical
/// one: the game sizes its interface from the logical window, half of these.
pub const RETINA: [Vec2; 4] = [
    Vec2::new(2560.0, 1440.0),
    Vec2::new(2880.0, 1800.0),
    Vec2::new(2940.0, 1912.0),
    Vec2::new(3456.0, 2234.0),
];

/// Physical pixels to one logical pixel on a Retina screen.
pub const RETINA_FACTOR: f32 = 2.0;

/// Returns every logical window the fit tests try: the named ones, the Retina ones, and windows
/// of every height from 540 to 1600 at the narrowest width and at common widths.
pub fn windows() -> Vec<Vec2> {
    let named = WINDOWS
        .into_iter()
        .chain(RETINA.map(|physical| physical / RETINA_FACTOR));
    let heights = (540..=1600).step_by(4).map(|height| height as f32);
    let swept = heights
        .flat_map(|height| [MIN_WIDTH, 1280.0, 1920.0].map(|width| Vec2::new(width, height)));
    named.chain(swept).collect()
}

/// Returns the size in logical pixels a page is laid out in, in a `window` at `size`: the window
/// over the interface scale.
pub fn layout(window: Vec2, size: InterfaceSize) -> Vec2 {
    window / ui_scale(window, size)
}

/// Returns the narrowest width any window of [`windows`] gives a page at any interface size.
pub fn narrowest() -> f32 {
    let each = windows()
        .into_iter()
        .flat_map(|window| InterfaceSize::ALL.map(|size| layout(window, size).x));
    each.fold(f32::INFINITY, f32::min)
}

/// Returns the lines that items `widths` wide take, `gap` apart, in lines `room` wide, laid out as
/// a wrapping row or a wrapping text lays them: an item goes to the next line when it does not fit
/// after the ones before it.
pub fn lines(widths: &[f32], gap: f32, room: f32) -> usize {
    let mut count = 0;
    let mut used = f32::INFINITY;
    for width in widths {
        if used + gap + width > room {
            count += 1;
            used = *width;
        } else {
            used += gap + width;
        }
    }
    count
}
