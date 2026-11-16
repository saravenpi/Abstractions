//! Colors and sizes shared by every menu and screen.

use bevy::prelude::*;

/// Game title text size.
pub const FONT_TITLE: f32 = 72.0;
/// Section heading text size.
pub const FONT_HEADING: f32 = 32.0;
/// Body and button text size.
pub const FONT_BODY: f32 = 20.0;

/// Spacing between stacked widgets.
pub const GAP: f32 = 10.0;
/// Inner padding of panels.
pub const PAD: f32 = 24.0;
/// Width of a menu button.
pub const BUTTON_WIDTH: f32 = 260.0;
/// Width of the small step buttons.
pub const STEP_WIDTH: f32 = 44.0;
/// Width of a settings label column.
pub const LABEL_WIDTH: f32 = 240.0;
/// Width of a settings value column.
pub const VALUE_WIDTH: f32 = 90.0;

/// Dimming layer behind a menu panel.
pub const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);
/// Panel background.
pub const PANEL: Color = Color::srgba(0.05, 0.06, 0.08, 0.82);
/// Idle button background.
pub const BUTTON: Color = Color::srgba(0.16, 0.18, 0.22, 0.9);
/// Hovered button background.
pub const BUTTON_HOVER: Color = Color::srgba(0.26, 0.30, 0.38, 0.95);
/// Pressed button background.
pub const BUTTON_PRESSED: Color = Color::srgba(0.36, 0.52, 0.62, 1.0);
/// Main text color.
pub const TEXT: Color = Color::srgb(0.92, 0.94, 0.96);
/// Muted text color.
pub const TEXT_DIM: Color = Color::srgb(0.62, 0.66, 0.72);
