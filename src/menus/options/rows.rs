//! Row widgets of the settings menu.

use bevy::prelude::*;

use super::actions::SettingsAction;
use super::field::{Bind, Field};
use crate::ui::{Icon, Tone, label, row_with, step_button, theme, value_cell};

/// Shows the live value of one setting.
#[derive(Component)]
pub struct Value(pub Field);

/// Muted setting name on the left of a row.
fn name(text: &str) -> impl Bundle {
    label(text, theme::FONT_BODY, Tone::Muted)
}

/// Live value text.
pub fn value(field: Field, size: f32) -> impl Bundle {
    (label("", size, Tone::Plain), Value(field))
}

/// Live value at the right end of the fixed value column.
fn ended(field: Field) -> impl Bundle {
    value_cell(JustifyContent::FlexEnd, value(field, theme::FONT_BODY))
}

/// Row with a value between minus and plus buttons.
pub fn stepper(text: &str, field: Field) -> impl Bundle {
    let cluster = (
        Node {
            align_items: AlignItems::Center,
            column_gap: px(theme::GAP_XS),
            ..default()
        },
        children![
            (
                step_button(Icon::ChevronLeft),
                SettingsAction::Adjust(field, -1)
            ),
            value_cell(JustifyContent::Center, value(field, theme::FONT_BODY)),
            (
                step_button(Icon::ChevronRight),
                SettingsAction::Adjust(field, 1)
            ),
        ],
    );
    (
        row_with(1, name(text), cluster),
        SettingsAction::Adjust(field, 1),
    )
}

/// Row whose value flips when activated.
pub fn toggle(text: &str, field: Field) -> impl Bundle {
    (
        row_with(1, name(text), ended(field)),
        SettingsAction::Adjust(field, 1),
    )
}

/// Row that shows the key of an action and waits for a new one when activated.
pub fn binding(bind: Bind) -> impl Bundle {
    (
        row_with(1, name(bind.label()), ended(Field::Key(bind))),
        SettingsAction::Rebind(bind),
    )
}

/// Row that puts every key, on foot and in the ship, back on its default when activated. The
/// defaults are physical keys, so they are the same places on every keyboard layout.
pub fn reset() -> impl Bundle {
    let what = label("To defaults", theme::FONT_BODY, Tone::Plain);
    (
        row_with(
            1,
            name("Reset all keys"),
            value_cell(JustifyContent::FlexEnd, what),
        ),
        SettingsAction::ResetKeys,
    )
}
