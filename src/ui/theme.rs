//! Palette, type scale and spacing scale shared by every menu and screen.
//!
//! The interface has no hue: white text and lines over the blurred, darkened game, and the
//! current row inverted, black on a white block. Nothing is rounded. The font is never bold:
//! hierarchy comes from size, case, spacing and inversion.
//!
//! Sizes are logical pixels. Layouts are built to fit a 960 by 540 window, the smallest supported.

use bevy::prelude::*;

/// Game title text size.
pub const FONT_TITLE: f32 = 64.0;
/// Section heading text size.
pub const FONT_HEADING: f32 = 28.0;
/// Body and menu item text size.
pub const FONT_BODY: f32 = 18.0;
/// Small label and hint text size.
pub const FONT_SMALL: f32 = 14.0;
/// Size of every icon: the 24 pixel grid the icons are drawn on, never scaled by a fraction.
pub const ICON: f32 = 24.0;
/// Letter spacing of uppercase section labels, in pixels.
pub const CAPS_SPACING: f32 = 3.0;
/// Advance of one glyph of the UI font, Undefined Medium, as a fraction of the font size.
pub const GLYPH_ASPECT: f32 = 0.6;
/// Most characters the value of a settings row may show. Every value is checked against it.
pub const VALUE_CHARS: usize = 14;
/// Width of the value column of a settings row: `VALUE_CHARS` glyphs of body text.
pub const VALUE_WIDTH: f32 = 152.0;

/// Base unit of the spacing scale. Every step below is a multiple of half of it.
pub const SPACE: f32 = 8.0;
/// Spacing step 1: inside a row, between a label and its key.
pub const GAP_XS: f32 = SPACE * 0.5;
/// Spacing step 2: inside rows and between small buttons.
pub const GAP_S: f32 = SPACE;
/// Spacing step 3: between a heading and its content.
pub const GAP_M: f32 = SPACE * 2.0;
/// Spacing step 4: between sections.
pub const GAP_L: f32 = SPACE * 3.0;
/// Spacing step 5: between columns and under a title.
pub const GAP_XL: f32 = SPACE * 5.0;

/// Width of a menu item column.
pub const MENU_WIDTH: f32 = 300.0;
/// Height of one settings or table row.
pub const ROW_HEIGHT: f32 = 32.0;
/// Height of one row of a dense table.
pub const ROW_COMPACT: f32 = 28.0;
/// Distance from the screen edge to the footer and corner texts.
pub const EDGE: f32 = 32.0;
/// Thickness of dividing lines.
pub const LINE: f32 = 1.0;

/// Share of the light of the blurred game that the backdrop of a menu removes.
///
/// It is set so that white text keeps a contrast of 4.5 to 1 over a pure white sky.
pub const TINT: f32 = 0.82;

/// Dark neutral background of screens with nothing behind them: title, loading, boot.
pub const BACKGROUND: Color = Color::srgb(0.03, 0.03, 0.03);
/// Backdrop of a menu when the blurred game is not available: the same darkening, without blur.
pub const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, TINT);
/// Backing strip of HUD texts over the live game.
pub const STRIP: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);
/// White text and lines.
pub const TEXT: Color = Color::srgb(1.0, 1.0, 1.0);
/// Secondary text: white at 65 percent.
pub const MUTED: Color = Color::srgba(1.0, 1.0, 1.0, 0.65);
/// Disabled text: white at 40 percent.
pub const FAINT: Color = Color::srgba(1.0, 1.0, 1.0, 0.4);
/// Dividing lines.
pub const HAIRLINE: Color = Color::srgba(1.0, 1.0, 1.0, 0.25);
/// The block behind the current row: solid white.
pub const BLOCK: Color = Color::srgb(1.0, 1.0, 1.0);
/// The block behind a row while it is held down.
pub const BLOCK_PRESSED: Color = Color::srgb(0.78, 0.78, 0.78);
/// Text on a white block: black.
pub const INK: Color = Color::srgb(0.0, 0.0, 0.0);
/// Background of a chosen tab, row or answer that does not hold the focus.
pub const SELECTED: Color = Color::srgba(1.0, 1.0, 1.0, 0.16);
/// Crosshair dot.
pub const CROSSHAIR: Color = Color::srgba(1.0, 1.0, 1.0, 0.8);

/// Every colour of the interface, for the test that none has a hue.
pub const PALETTE: [Color; 12] = [
    BACKGROUND,
    SCRIM,
    STRIP,
    TEXT,
    MUTED,
    FAINT,
    HAIRLINE,
    BLOCK,
    BLOCK_PRESSED,
    INK,
    SELECTED,
    CROSSHAIR,
];
