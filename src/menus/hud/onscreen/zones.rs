//! The room the markers leave the rest of the HUD: a safe rectangle for their icons, clear of
//! the compass row at the top and of the prompt and the hotbar at the bottom, and the corners
//! they move out of, where the place lines, the minimap and its caption and the instruments sit.
//!
//! Everything is in interface pixels, so the room follows the "Interface size" setting.

use bevy::prelude::*;

use super::tune::{ARROW_REACH, HALF};
use crate::craft::ui::hotbar;
use crate::menus::hud::compass::HEIGHT;
use crate::menus::hud::minimap::widget;
use crate::ui::theme;

/// Width and height in pixels of the place lines, top left, at their widest.
const PLACE: Vec2 = Vec2::new(272.0, 72.0);

/// Width and height in pixels of the instruments and the frame rows, bottom left, in the ship.
const DECK: Vec2 = Vec2::new(336.0, 280.0);

/// Height in pixels of the prompt row above the hotbar.
const PROMPT: f32 = 36.0;

/// Returns the rectangle the centre of every icon stays in on a screen of `screen` pixels: in
/// from the sides by the edge margin, under the compass row, above the prompt and the hotbar,
/// each with the room of the arrow or of the words.
pub fn safe(screen: Vec2) -> Rect {
    let side = theme::EDGE + ARROW_REACH + theme::ICON * 0.5 + theme::S1 * 0.5;
    let top = theme::EDGE + HEIGHT + theme::S2 + HALF.1;
    let below = theme::EDGE + hotbar::SIDE + theme::S2 + PROMPT + theme::S2 + HALF.1;
    let (min, max) = (Vec2::new(side, top), screen - Vec2::new(side, below));
    Rect::from_corners(min, max.max(min))
}

/// Returns the corners of a screen of `screen` pixels the markers keep out of: the place lines
/// top left, the minimap and its caption top right and, while `seated`, the instruments bottom
/// left.
pub fn corners(screen: Vec2, seated: bool) -> [Rect; 3] {
    let gap = theme::EDGE + theme::S2;
    let map = (screen.y * widget::SIDE / 1080.0).clamp(widget::LIMITS.0, widget::LIMITS.1);
    let deck = DECK * f32::from(u8::from(seated));
    [
        Rect::from_corners(Vec2::ZERO, PLACE + gap),
        Rect::new(
            screen.x - map - gap,
            0.0,
            screen.x,
            map + widget::CAPTION + gap,
        ),
        Rect::new(0.0, screen.y - deck.y, deck.x, screen.y),
    ]
}

/// Returns `pos` moved out of `zone` by the shortest way that stays inside `safe`, or `pos`
/// itself when its marker, [`HALF`] pixels to each side, is clear of the zone already.
fn out_of(pos: Vec2, zone: Rect, safe: Rect) -> Vec2 {
    let half = Vec2::new(HALF.0, HALF.1);
    let grown = Rect::from_corners(zone.min - half, zone.max + half);
    let inside = pos.cmpgt(grown.min).all() && pos.cmplt(grown.max).all();
    if zone.is_empty() || !inside {
        return pos;
    }
    let exits = [
        Vec2::new(grown.min.x, pos.y),
        Vec2::new(grown.max.x, pos.y),
        Vec2::new(pos.x, grown.min.y),
        Vec2::new(pos.x, grown.max.y),
    ];
    let near = |a: &Vec2, b: &Vec2| a.distance(pos).total_cmp(&b.distance(pos));
    let open = exits.into_iter().filter(|exit| safe.contains(*exit));
    open.min_by(near).unwrap_or(pos)
}

/// Returns `pos`, the centre of an icon, moved so its marker covers no corner of a screen of
/// `screen` pixels that the HUD uses, `seated` or not.
pub fn fit(pos: Vec2, screen: Vec2, seated: bool) -> Vec2 {
    let safe = safe(screen);
    let zones = corners(screen, seated);
    let once = |pos: Vec2| zones.iter().fold(pos, |pos, zone| out_of(pos, *zone, safe));
    once(once(safe.clamp_point(pos)))
}
