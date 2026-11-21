//! Colour tokens, type scale and spacing scale shared by every menu and screen.
//!
//! The owner's direction of 2026-11-20: square corners everywhere, no borders, the Undefined
//! font only and never bold, translucent panels, one restrained accent system. Hierarchy comes
//! from size, tone, space and the one accent. Separate things with space, tone and translucency.
//!
//! Sizes are logical pixels at the 1280 by 720 reference. `UiScale` (see `ui::camera`) scales
//! the whole interface with the window height, so a layout is designed once.

use bevy::prelude::*;

/// Title screen text size.
pub const FONT_TITLE: f32 = 72.0;
/// Line height of [`FONT_TITLE`].
pub const LINE_TITLE: f32 = 80.0;
/// Banner and full-screen menu heading size.
pub const FONT_HEADING: f32 = 32.0;
/// Line height of [`FONT_HEADING`].
pub const LINE_HEADING: f32 = 40.0;
/// Card title, primary button, prompt warning and menu item size.
pub const FONT_BODY: f32 = 20.0;
/// Line height of [`FONT_BODY`].
pub const LINE_BODY: f32 = 28.0;
/// Size of all HUD text, list rows, facts, hints and key caps. The smallest text anywhere.
pub const FONT_SMALL: f32 = 16.0;
/// Line height of [`FONT_SMALL`].
pub const LINE_SMALL: f32 = 24.0;
/// Size of every icon: the 24 pixel grid the icons are drawn on, never scaled by a fraction.
pub const ICON: f32 = 24.0;
/// Icon size used only inline in a 16 pixel text line.
pub const ICON_SMALL: f32 = 16.0;
/// Letter spacing of uppercase section labels, in pixels.
pub const CAPS_SPACING: f32 = 3.0;
/// Advance of one glyph of the UI font, Undefined Medium, as a fraction of the font size.
pub const GLYPH_ASPECT: f32 = 0.6;
/// Most characters the value of a settings row may show. Every value is checked against it.
pub const VALUE_CHARS: usize = 14;
/// Width of the value column of a settings row: `VALUE_CHARS` glyphs of body text.
pub const VALUE_WIDTH: f32 = 168.0;

/// Spacing step 1: between an icon and its text, between lines of one fact.
pub const S1: f32 = 4.0;
/// Spacing step 2: between small siblings, such as a key cap and its action, and chips.
pub const S2: f32 = 8.0;
/// Spacing step 3: horizontal padding of a one-line strip, between rows of a list.
pub const S3: f32 = 16.0;
/// Spacing step 4: padding of a panel on all sides, gap between groups inside a panel.
pub const S4: f32 = 24.0;
/// Spacing step 5: screen edge margin on every side, gap between two panels.
pub const S5: f32 = 32.0;
/// Spacing step 6: between sections of a full-screen menu, top margin under a title.
pub const S6: f32 = 48.0;

/// Old name of [`S1`].
pub const GAP_XS: f32 = S1;
/// Old name of [`S2`].
pub const GAP_S: f32 = S2;
/// Old name of [`S3`].
pub const GAP_M: f32 = S3;
/// Old name of [`S4`].
pub const GAP_L: f32 = S4;
/// Old name of [`S6`].
pub const GAP_XL: f32 = S6;

/// Width of a menu item column.
pub const MENU_WIDTH: f32 = 300.0;
/// Height of one settings or table row. Also the smallest hit target.
pub const ROW_HEIGHT: f32 = 32.0;
/// Height of one row of a dense table.
pub const ROW_COMPACT: f32 = 28.0;
/// Distance from the screen edge to every panel, corner text and the prompt line: [`S5`].
pub const EDGE: f32 = S5;
/// Thickness of thin rules drawn as filled nodes. Not a border.
pub const LINE: f32 = 1.0;

/// Share of the light of the blurred game that the backdrop of a menu removes.
///
/// It is set so that white text keeps a contrast of 4.5 to 1 over a pure white sky.
pub const TINT: f32 = 0.82;

/// Dark neutral background of screens with nothing behind them: title, loading, boot.
pub const BACKGROUND: Color = Color::srgb(0.03, 0.03, 0.03);
/// Backdrop of a menu when the blurred game is not available: the same darkening, without blur.
pub const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, TINT);
/// Primary text.
pub const TEXT: Color = Color::srgb(1.0, 1.0, 1.0);
/// Secondary text and labels: white at 72 percent.
pub const TEXT_2: Color = Color::srgba(1.0, 1.0, 1.0, 0.72);
/// Disabled text only: white at 45 percent. It never carries information alone.
pub const TEXT_OFF: Color = Color::srgba(1.0, 1.0, 1.0, 0.45);
/// Text on an accent fill.
pub const INK: Color = Color::srgb(0.02, 0.03, 0.04);
/// Fill of every HUD panel and strip over the live world: a light dark tint at 62 percent. Over a
/// pure white sky, the worst case, `TEXT` keeps 5.8 to 1, `TEXT_2` 3.9 to 1, `PIN` 4.3 to 1 and
/// `ACCENT` 3.2 to 1. Text in `WARN` and `DANGER` does not reach 3 to 1 on it: rows that can
/// hold it sit on [`PANEL_DEEP`].
pub const PANEL: Color = Color::srgba(0.02, 0.025, 0.035, 0.62);
/// Fill of the rows that can hold `WARN` or `DANGER` text over the live world: the darker tint
/// that keeps `DANGER` at 5 to 1 over a white sky and 7 to 1 over a black one.
pub const PANEL_DEEP: Color = Color::srgba(0.02, 0.025, 0.035, 0.84);
/// Fill of a big blurred panel, laid over the `Blurred` backdrop.
pub const PANEL_BLUR: Color = Color::srgba(0.04, 0.05, 0.07, 0.55);
/// Row under the pointer or the keyboard focus.
pub const HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);
/// Chosen row: the accent at 20 percent. A 4 pixel bar of [`ACCENT`] marks its left edge.
pub const SELECT: Color = Color::srgba(0.40, 0.80, 1.0, 0.20);
/// The one accent: the primary action, selection, the course line and its compass marker.
pub const ACCENT: Color = Color::srgb(0.40, 0.80, 1.0);
/// Pins: waypoints and favourites, one kind.
pub const PIN: Color = Color::srgb(1.0, 0.86, 0.35);
/// A blocked course, weather and heat warnings, frame integrity under 50 percent.
pub const WARN: Color = Color::srgb(1.0, 0.60, 0.25);
/// Frame integrity under 25 percent, a broken part.
pub const DANGER: Color = Color::srgb(1.0, 0.40, 0.38);

/// Dividing lines.
pub const HAIRLINE: Color = Color::srgba(1.0, 1.0, 1.0, 0.25);
/// Crosshair dot.
pub const CROSSHAIR: Color = Color::srgba(1.0, 1.0, 1.0, 0.8);

/// Every translucent backdrop that lets the game show through.
pub const TRANSLUCENT: [Color; 6] = [SCRIM, PANEL, PANEL_DEEP, PANEL_BLUR, HOVER, SELECT];
