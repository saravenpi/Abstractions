//! Palette, type scale and spacing shared by every menu and screen.

use bevy::prelude::*;

/// Game title text size.
pub const FONT_TITLE: f32 = 64.0;
/// Section heading text size.
pub const FONT_HEADING: f32 = 28.0;
/// Body and menu item text size.
pub const FONT_BODY: f32 = 18.0;
/// Small label and hint text size.
pub const FONT_SMALL: f32 = 14.0;
/// Letter spacing of uppercase section labels, in pixels.
pub const CAPS_SPACING: f32 = 3.0;
/// Base unit of the spacing grid.
pub const SPACE: f32 = 8.0;
/// Width of a menu item column.
pub const MENU_WIDTH: f32 = 300.0;
/// Height of one settings row.
pub const ROW_HEIGHT: f32 = 32.0;
/// Horizontal shift of a highlighted menu item.
pub const SHIFT: f32 = 6.0;
/// Distance from the screen edge to the footer texts.
pub const EDGE: f32 = 32.0;

/// Solid ink background of full screens.
pub const BACKGROUND: Color = Color::srgb(0.0549, 0.0667, 0.0863);
/// Ink overlay laid over the world behind menus.
pub const OVERLAY: Color = Color::srgba(0.0549, 0.0667, 0.0863, 0.72);
/// Off-white main text.
pub const TEXT: Color = Color::srgb(0.9255, 0.9098, 0.8745);
/// Off-white text of an idle menu item.
pub const ITEM: Color = Color::srgba(0.9255, 0.9098, 0.8745, 0.72);
/// Off-white text of a pressed menu item.
pub const ITEM_PRESSED: Color = Color::srgba(0.9255, 0.9098, 0.8745, 0.5);
/// Muted grey text.
pub const MUTED: Color = Color::srgb(0.5412, 0.5608, 0.5961);
/// Warm sand accent.
pub const ACCENT: Color = Color::srgb(0.8863, 0.7216, 0.4);
/// Idle accent text.
pub const ACCENT_REST: Color = Color::srgba(0.8863, 0.7216, 0.4, 0.85);
/// Pressed accent text.
pub const ACCENT_PRESSED: Color = Color::srgba(0.8863, 0.7216, 0.4, 0.6);
/// Hairline rules.
pub const HAIRLINE: Color = Color::srgba(1.0, 1.0, 1.0, 0.12);
/// Crosshair dot.
pub const CROSSHAIR: Color = Color::srgba(0.9255, 0.9098, 0.8745, 0.7);
