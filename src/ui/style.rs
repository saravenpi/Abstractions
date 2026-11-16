//! Hover and pressed colors for buttons.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::Button;

use super::theme;

/// Colors every button by its hovered and pressed state.
pub(super) fn paint_buttons(
    mut buttons: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<Button>>,
) {
    for (hovered, pressed, mut background) in &mut buttons {
        let color = match (pressed, hovered.get()) {
            (true, _) => theme::BUTTON_PRESSED,
            (false, true) => theme::BUTTON_HOVER,
            _ => theme::BUTTON,
        };
        if background.0 != color {
            background.0 = color;
        }
    }
}
